use minifb::{Key, Window, WindowOptions};
use rand::Rng;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;
const STAR_COUNT: usize = 180;      // 大幅减少数量，更稀疏
const METEOR_CHANCE: f32 = 0.008;   // 每帧大约 0.8% 概率产生一颗流星

struct Star {
    x: f32,
    y: f32,
    z: f32,           // 控制远近（越小越远）
    twinkle_phase: f32, // 用于轻微闪烁
}

struct Meteor {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    life: f32,        // 生命值，逐渐减少
    max_life: f32,
}

fn main() {
    let mut rng = rand::thread_rng();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Starfield - ESC to quit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    // 生成稀疏的背景星星
    let mut stars: Vec<Star> = (0..STAR_COUNT)
        .map(|_| Star {
            x: rng.gen_range(0.0..WIDTH as f32),
            y: rng.gen_range(0.0..HEIGHT as f32),
            z: rng.gen_range(1.2..6.0),     // 大部分较远
            twinkle_phase: rng.gen_range(0.0..10.0),
        })
        .collect();

    let mut meteors: Vec<Meteor> = Vec::new();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600))); // ~60fps

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0);

        // 1. 绘制普通星星（稀疏 + 轻微闪烁）
        for star in stars.iter_mut() {
            // 非常缓慢的漂移（几乎静止感，但有一点点动）
            star.x += (star.x - WIDTH as f32 / 2.0) * 0.0003 * (6.0 - star.z);
            star.y += (star.y - HEIGHT as f32 / 2.0) * 0.0003 * (6.0 - star.z);

            // 轻微边界回绕（避免完全静止）
            if star.x < 0.0 { star.x += WIDTH as f32; }
            if star.x > WIDTH as f32 { star.x -= WIDTH as f32; }
            if star.y < 0.0 { star.y += HEIGHT as f32; }
            if star.y > HEIGHT as f32 { star.y -= HEIGHT as f32; }

            // 闪烁
            star.twinkle_phase += 0.03;
            let brightness = (120.0 + (star.twinkle_phase.sin() * 50.0)) as u32;
            let size = if star.z > 4.0 { 0.8 } else { 1.2 };

            let sx = star.x as isize;
            let sy = star.y as isize;

            if sx >= 0 && sx < WIDTH as isize && sy >= 0 && sy < HEIGHT as isize {
                let idx = sy as usize * WIDTH + sx as usize;
                let color = (brightness << 16) | (brightness << 8) | brightness;
                buffer[idx] = color;

                // 轻微光晕（只给较亮的星星）
                if brightness > 140 && size > 1.0 {
                    if sx > 0 { buffer[sy as usize * WIDTH + (sx - 1) as usize] = color / 3; }
                    if sx < WIDTH as isize - 1 { buffer[sy as usize * WIDTH + (sx + 1) as usize] = color / 3; }
                }
            }
        }

        // 2. 随机产生流星（少量）
        if rng.gen::<f32>() < METEOR_CHANCE {
            let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let speed = rng.gen_range(8.0..16.0);
            let len = rng.gen_range(60.0..140.0);

            meteors.push(Meteor {
                x: if angle < std::f32::consts::PI { -20.0 } else { WIDTH as f32 + 20.0 },
                y: rng.gen_range(-50.0..HEIGHT as f32 + 50.0),
                dx: speed * angle.cos(),
                dy: speed * angle.sin(),
                life: len,
                max_life: len,
            });
        }

        // 3. 更新并绘制流星
        meteors.retain_mut(|m| {
            m.x += m.dx;
            m.y += m.dy;
            m.life -= 1.0;

            if m.life <= 0.0 {
                return false;
            }

            let alpha = (m.life / m.max_life * 220.0) as u32;
            let tail_steps = (m.max_life.min(100.0)) as usize;

            for i in 0..=tail_steps {
                if i as f32 > m.life { break; }
                let factor = 1.0 - (i as f32 / tail_steps as f32);
                let bright = (alpha as f32 * factor) as u32;

                let tx = (m.x - m.dx * i as f32 * 0.6) as isize;
                let ty = (m.y - m.dy * i as f32 * 0.6) as isize;

                if tx >= 0 && tx < WIDTH as isize && ty >= 0 && ty < HEIGHT as isize {
                    let idx = ty as usize * WIDTH + tx as usize;
                    buffer[idx] = (bright << 16) | (bright << 8) | (bright / 2 + 40); // 偏白带点蓝
                }
            }

            true
        });

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}