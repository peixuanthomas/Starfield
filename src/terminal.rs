use crate::scene::{Scene, HEIGHT, WIDTH};
use base64::{engine::general_purpose::STANDARD, Engine};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Color, ResetColor, SetBackgroundColor},
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use flate2::{write::ZlibEncoder, Compression};
use std::{
    env,
    io::{self, IsTerminal, Write},
    time::{Duration, Instant},
};

const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);
// Reuse two image IDs so the previous frame stays visible during the next upload.
const IMAGE_IDS: [u32; 2] = [31_415, 31_416];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Auto,
    Kitty,
    Ansi,
}

impl Backend {
    fn resolve(self) -> Self {
        if self != Self::Auto {
            return self;
        }
        let term = env::var("TERM").unwrap_or_default();
        let program = env::var("TERM_PROGRAM").unwrap_or_default();
        // Multiplexers can retain the outer terminal's environment while blocking
        // image commands. Prefer the compatible path unless explicitly overridden.
        let multiplexer = env::var_os("TMUX").is_some()
            || env::var_os("STY").is_some()
            || term.starts_with("screen")
            || term.starts_with("tmux");
        if !multiplexer && (term == "xterm-kitty" || program == "ghostty") {
            Self::Kitty
        } else {
            Self::Ansi
        }
    }
}

/// Restore the shell on normal exit, I/O errors and panic unwinding.
struct Session {
    backend: Backend,
}

impl Session {
    fn enter(backend: Backend) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        // Create the guard before any fallible screen changes.
        let session = Self { backend };
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            DisableLineWrap,
            Hide,
            SetBackgroundColor(Color::Black),
            Clear(ClearType::All)
        )?;
        Ok(session)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let mut out = io::stdout();
        // Terminate any interrupted APC transfer before issuing cleanup commands.
        let _ = out.write_all(b"\x1b\\");
        if self.backend == Backend::Kitty {
            for id in IMAGE_IDS {
                let _ = write!(out, "\x1b_Ga=d,d=I,i={id},q=2;\x1b\\");
            }
        }
        let _ = execute!(out, ResetColor, EnableLineWrap, Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Viewport {
    x: u16,
    y: u16,
    columns: u16,
    rows: u16,
}

impl Viewport {
    fn current() -> io::Result<Self> {
        let (columns, rows) = terminal::size()?;
        let cell_aspect = terminal::window_size()
            .ok()
            .filter(|s| s.width > 0 && s.height > 0 && s.columns > 0 && s.rows > 0)
            .map(|s| (s.width as f64 / s.columns as f64) / (s.height as f64 / s.rows as f64))
            .unwrap_or(0.5); // Typical terminal cell is twice as tall as it is wide.
        Ok(Self::fit(columns, rows, cell_aspect))
    }

    fn fit(columns: u16, rows: u16, cell_aspect: f64) -> Self {
        if columns == 0 || rows == 0 {
            return Self {
                x: 0,
                y: 0,
                columns: 0,
                rows: 0,
            };
        }
        let ratio = WIDTH as f64 / HEIGHT as f64 / cell_aspect;
        let width = (rows as f64 * ratio).floor().clamp(1.0, columns as f64) as u16;
        let height = (width as f64 / ratio).round().clamp(1.0, rows as f64) as u16;
        Self {
            x: (columns - width) / 2,
            y: (rows - height) / 2,
            columns: width,
            rows: height,
        }
    }
}

fn is_quit(key: KeyEvent) -> bool {
    key.kind != KeyEventKind::Release
        && (matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')
        ) || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)))
}

pub fn run(backend: Backend) -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(io::Error::other(
            "Terminal mode requires an interactive stdin and stdout.",
        ));
    }
    if env::var("TERM").as_deref() == Ok("dumb") {
        return Err(io::Error::other(
            "TERM=dumb does not support terminal rendering.",
        ));
    }
    let backend = backend.resolve();
    let _session = Session::enter(backend)?;
    let mut out = io::stdout().lock();
    let mut viewport = Viewport::current()?;
    let mut rng = rand::thread_rng();
    let mut scene = Scene::new(&mut rng);
    let mut pixels = vec![0; WIDTH * HEIGHT];
    let mut frame = Vec::new();
    let mut kitty = KittyRenderer::default();
    let mut next_tick = Instant::now();

    loop {
        let now = Instant::now();
        if now >= next_tick {
            // Keep animation speed tied to 60 Hz even if terminal output is slower.
            // Bound catch-up after a suspension to avoid an unresponsive loop.
            for _ in 0..5 {
                scene.tick(&mut rng, &mut pixels);
                next_tick += FRAME_TIME;
                if next_tick > now {
                    break;
                }
            }
            if next_tick <= now {
                next_tick = now + FRAME_TIME;
            }
            if viewport.columns > 0 && viewport.rows > 0 {
                frame.clear();
                match backend {
                    Backend::Kitty => kitty.encode(&mut frame, &pixels, viewport)?,
                    Backend::Ansi => encode_ansi(&mut frame, &pixels, viewport)?,
                    Backend::Auto => unreachable!(),
                }
                out.write_all(&frame)?;
                out.flush()?;
            }
        }
        if event::poll(next_tick.saturating_duration_since(Instant::now()))? {
            match event::read()? {
                Event::Key(key) if is_quit(key) => break,
                Event::Resize(_, _) => {
                    viewport = Viewport::current()?;
                    execute!(out, SetBackgroundColor(Color::Black), Clear(ClearType::All))?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[derive(Default)]
struct KittyRenderer {
    rgb: Vec<u8>,
    image_index: usize,
}

impl KittyRenderer {
    fn encode(&mut self, out: &mut Vec<u8>, pixels: &[u32], view: Viewport) -> io::Result<()> {
        self.rgb.clear();
        self.rgb.reserve(pixels.len() * 3);
        for pixel in pixels {
            self.rgb
                .extend_from_slice(&[(pixel >> 16) as u8, (pixel >> 8) as u8, *pixel as u8]);
        }
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&self.rgb)?;
        let payload = STANDARD.encode(encoder.finish()?);
        let current = IMAGE_IDS[self.image_index];
        let previous = IMAGE_IDS[1 - self.image_index];
        // Direct transmission works over SSH without sharing files with the terminal.
        // Protocol: https://sw.kovidgoyal.net/kitty/graphics-protocol/
        let chunks = payload.as_bytes().chunks(4096);
        let count = chunks.len();
        for (index, chunk) in chunks.enumerate() {
            let more = u8::from(index + 1 < count);
            if index == 0 {
                write!(
                    out,
                    "\x1b_Ga=t,t=d,f=24,s={WIDTH},v={HEIGHT},o=z,i={current},q=2,m={more};"
                )?;
            } else {
                write!(out, "\x1b_Gm={more};")?;
            }
            out.write_all(chunk)?;
            out.write_all(b"\x1b\\")?;
        }
        queue!(out, MoveTo(view.x, view.y))?;
        write!(
            out,
            "\x1b_Ga=p,i={current},p=1,c={},r={},C=1,q=2;\x1b\\",
            view.columns, view.rows
        )?;
        // Only free the old frame after the new one is complete and placed.
        write!(out, "\x1b_Ga=d,d=I,i={previous},q=2;\x1b\\")?;
        self.image_index = 1 - self.image_index;
        Ok(())
    }
}

fn brightness(pixel: u32) -> u32 {
    ((pixel >> 16) & 255) + ((pixel >> 8) & 255) + (pixel & 255)
}

/// Retain the brightest source pixel in each output bin. Averaging or nearest
/// sampling would erase almost all the original single-pixel stars when shrinking.
fn sample_pixels(pixels: &[u32], width: usize, height: usize) -> Vec<u32> {
    let mut sampled = vec![0; width * height];
    for y in 0..height {
        let y0 = y * HEIGHT / height;
        let y1 = ((y + 1) * HEIGHT / height).max(y0 + 1).min(HEIGHT);
        for x in 0..width {
            let x0 = x * WIDTH / width;
            let x1 = ((x + 1) * WIDTH / width).max(x0 + 1).min(WIDTH);
            let mut best = 0;
            for sy in y0..y1 {
                for &pixel in &pixels[sy * WIDTH + x0..sy * WIDTH + x1] {
                    if brightness(pixel) > brightness(best) {
                        best = pixel;
                    }
                }
            }
            sampled[y * width + x] = best;
        }
    }
    sampled
}

fn encode_ansi(out: &mut Vec<u8>, pixels: &[u32], view: Viewport) -> io::Result<()> {
    let width = view.columns as usize;
    let samples = sample_pixels(pixels, width, view.rows as usize * 2);
    let mut colors = None;
    for row in 0..view.rows {
        queue!(out, MoveTo(view.x, view.y + row))?;
        for column in 0..width {
            let top = samples[row as usize * 2 * width + column];
            let bottom = samples[(row as usize * 2 + 1) * width + column];
            if colors != Some((top, bottom)) {
                write!(
                    out,
                    "\x1b[38;2;{};{};{};48;2;{};{};{}m",
                    (top >> 16) & 255,
                    (top >> 8) & 255,
                    top & 255,
                    (bottom >> 16) & 255,
                    (bottom >> 8) & 255,
                    bottom & 255
                )?;
                colors = Some((top, bottom));
            }
            out.write_all("▀".as_bytes())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
