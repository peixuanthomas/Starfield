use minifb::{Key, Window, WindowOptions};
use rand::Rng;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;

struct Star {
    x: f32,
    y: f32,
    z: f32,        // 用来控制远近（决定大小+速度）
    speed: f32,
    brightness: u32,
}

fn main() {
    let mut rng = rand::thread_rng();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Starfield - Press ESC to quit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // 生成星星
    let mut stars: Vec<Star> = (0..800)
        .map(|_| Star {
            x: rng.gen_range(0.0..WIDTH as f32),
            y: rng.gen_range(0.0..HEIGHT as f32),
            z: rng.gen_range(0.1..5.0),     // z越小越远
            speed: rng.gen_range(0.3..1.8),
            brightness: rng.gen_range(80..255) as u32,
        })
        .collect();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600))); // ~60fps

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // 清空画面（纯黑）
        buffer.fill(0);

        for star in stars.iter_mut() {
            // 星星向外漂移（模拟3D向外扩散的感觉）
            star.x += (star.x - WIDTH as f32 / 2.0) * star.speed * 0.004 * star.z;
            star.y += (star.y - HEIGHT as f32 / 2.0) * star.speed * 0.004 * star.z;
            star.z += star.speed * 0.008; // 慢慢变近

            // 超出画面 → 重置到中心附近（制造无限感）
            if star.x < -50.0
                || star.x > WIDTH as f32 + 50.0
                || star.y < -50.0
                || star.y > HEIGHT as f32 + 50.0
                || star.z > 8.0
            {
                star.x = WIDTH as f32 / 2.0 + rng.gen_range(-60.0..60.0);
                star.y = HEIGHT as f32 / 2.0 + rng.gen_range(-60.0..60.0);
                star.z = rng.gen_range(0.1..1.5);
                star.speed = rng.gen_range(0.4..1.9);
                star.brightness = rng.gen_range(90..255) as u32;
            }

            // 根据距离计算大小和亮度
            let size_factor = (6.0 - star.z).max(0.6);
            let brightness = ((star.brightness as f32 * (5.0 - star.z) / 5.0) as u32).min(255);

            // 画星星（简单十字形/点）
            let sx = star.x as isize;
            let sy = star.y as isize;

            if sx >= 0 && sx < WIDTH as isize && sy >= 0 && sy < HEIGHT as isize {
                let idx = (sy as usize * WIDTH) + sx as usize;
                buffer[idx] = (brightness << 16) | (brightness << 8) | brightness;

                // 稍微画大一点点（十字）
                if size_factor > 1.2 {
                    if sx > 0 {
                        buffer[(sy as usize * WIDTH) + (sx - 1) as usize] =
                            ((brightness / 2) << 16) | ((brightness / 2) << 8) | (brightness / 2);
                    }
                    if sx < WIDTH as isize - 1 {
                        buffer[(sy as usize * WIDTH) + (sx + 1) as usize] =
                            ((brightness / 2) << 16) | ((brightness / 2) << 8) | (brightness / 2);
                    }
                    if sy > 0 {
                        buffer[((sy - 1) as usize * WIDTH) + sx as usize] =
                            ((brightness / 2) << 16) | ((brightness / 2) << 8) | (brightness / 2);
                    }
                    if sy < HEIGHT as isize - 1 {
                        buffer[((sy + 1) as usize * WIDTH) + sx as usize] =
                            ((brightness / 2) << 16) | ((brightness / 2) << 8) | (brightness / 2);
                    }
                }
            }
        }

        // 更新窗口
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}