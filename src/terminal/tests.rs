use super::*;
use flate2::read::ZlibDecoder;
use rand::{rngs::StdRng, SeedableRng};
use std::io::Read;

#[test]
fn kitty_transmission_preserves_every_scene_pixel() {
    let mut rng = StdRng::seed_from_u64(123);
    let mut scene = Scene::new(&mut rng);
    let mut pixels = vec![0; WIDTH * HEIGHT];
    let mut renderer = KittyRenderer::default();
    let view = Viewport::fit(120, 40, 0.5);
    // Include twinkling stars, a moving meteor, and multiple image replacements.
    for frame_number in 0..4 {
        for _ in 0..30 {
            scene.tick(&mut rng, &mut pixels);
        }
        let mut encoded = Vec::new();
        renderer.encode(&mut encoded, &pixels, view).unwrap();
        let text = String::from_utf8(encoded).unwrap();
        let mut payload = String::new();
        let mut chunks = 0;
        let mut finished = false;
        for command in text.split("\x1b_G").skip(1) {
            let command = command.split("\x1b\\").next().unwrap();
            let (control, data) = command.split_once(';').unwrap();
            if control.starts_with("a=t,") || control.starts_with("m=") {
                assert!(!finished);
                assert!(data.len() <= 4096);
                assert_eq!(data.len() % 4, 0);
                payload.push_str(data);
                finished = control.ends_with("m=0");
                chunks += 1;
            }
        }
        assert!(chunks > 1, "Exercise the chunked transfer path");
        assert!(finished);
        let current = IMAGE_IDS[frame_number % 2];
        let previous = IMAGE_IDS[1 - frame_number % 2];
        assert!(text.contains(&format!("f=24,s={WIDTH},v={HEIGHT},o=z,i={current},q=2")));
        assert!(text.contains(&format!("a=p,i={current},p=1,c=120,r=40,C=1,q=2")));
        assert!(text.ends_with(&format!("\x1b_Ga=d,d=I,i={previous},q=2;\x1b\\")));
        let compressed = STANDARD.decode(payload).unwrap();
        let mut decoded = Vec::new();
        ZlibDecoder::new(compressed.as_slice())
            .read_to_end(&mut decoded)
            .unwrap();
        assert_eq!(decoded.len(), WIDTH * HEIGHT * 3);
        for (index, &pixel) in pixels.iter().enumerate() {
            let rgb = &decoded[index * 3..index * 3 + 3];
            assert_eq!(rgb, &[(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8]);
        }
    }
}

#[test]
fn ansi_downsampling_keeps_single_pixel_stars_and_meteor_colors() {
    let mut pixels = vec![0; WIDTH * HEIGHT];
    pixels[0] = 0xababab;
    pixels[100 * WIDTH + 200] = 0xeeeeaa;
    pixels[HEIGHT * WIDTH - 1] = 0x556699;
    let sampled = sample_pixels(&pixels, 120, 80);
    assert_eq!(sampled[0], 0xababab);
    assert_eq!(sampled[10 * 120 + 20], 0xeeeeaa);
    assert_eq!(sampled[80 * 120 - 1], 0x556699);
    assert_eq!(sampled.iter().filter(|&&p| p != 0).count(), 3);
    // Resizing to a larger grid must not index beyond the source frame.
    let enlarged = sample_pixels(&pixels, WIDTH + 1, HEIGHT + 1);
    assert_eq!(enlarged.last(), Some(&0x556699));
}

#[test]
fn ansi_encodes_top_and_bottom_colors_without_scrolling() {
    let mut pixels = vec![0; WIDTH * HEIGHT];
    pixels[0] = 0x123456;
    pixels[(HEIGHT / 2) * WIDTH] = 0xaabbcc;
    let mut encoded = Vec::new();
    encode_ansi(
        &mut encoded,
        &pixels,
        Viewport {
            x: 2,
            y: 3,
            columns: 1,
            rows: 1,
        },
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(encoded).unwrap(),
        "\x1b[4;3H\x1b[38;2;18;52;86;48;2;170;187;204m▀"
    );
}

#[test]
fn blank_ansi_frame_erases_previous_stars() {
    let mut encoded = Vec::new();
    encode_ansi(
        &mut encoded,
        &vec![0; WIDTH * HEIGHT],
        Viewport::fit(6, 2, 0.5),
    )
    .unwrap();
    let text = String::from_utf8(encoded).unwrap();
    assert!(text.contains("38;2;0;0;0;48;2;0;0;0m"));
    assert_eq!(text.matches('▀').count(), 12);
}

#[test]
fn viewport_preserves_aspect_and_stays_inside_small_and_resized_terminals() {
    for (columns, rows, aspect) in [
        (120, 40, 0.5),
        (80, 24, 0.5),
        (200, 20, 0.5),
        (30, 90, 0.6),
        (1, 1, 0.5),
    ] {
        let view = Viewport::fit(columns, rows, aspect);
        assert!(view.columns > 0 && view.rows > 0);
        assert!(view.x + view.columns <= columns);
        assert!(view.y + view.rows <= rows);
        // Allow at most one terminal row of quantization error.
        let target_rows = view.columns as f64 * aspect * HEIGHT as f64 / WIDTH as f64;
        assert!((view.rows as f64 - target_rows).abs() <= 1.0);
    }
    assert_eq!(Viewport::fit(0, 0, 0.5).columns, 0);
    assert_eq!(Viewport::fit(80, 0, 0.5).rows, 0);
}

#[test]
fn terminal_quit_keys_ignore_key_release_events() {
    for key in [
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
    ] {
        assert!(is_quit(key));
        assert!(!is_quit(KeyEvent {
            kind: KeyEventKind::Release,
            ..key
        }));
    }
    assert!(!is_quit(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::NONE
    )));
}
