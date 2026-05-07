use minifb::{Key, Window, WindowOptions};
use rand::Rng;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;
const STAR_COUNT: usize = 180; // 大幅减少数量，更稀疏
const METEOR_CHANCE: f32 = 0.008; // 每帧大约 0.8% 概率产生一颗流星
const STAR_DRIFT: f32 = 0.0003;
const STAR_SPAWN_MARGIN: f32 = 12.0;
const TWINKLE_SPEED: f32 = 0.03;
const METEOR_FORCE_AFTER_FRAMES: usize = 360; // 最多约 6 秒保证出现一颗流星

struct Star {
    x: f32,
    y: f32,
    z: f32,             // 控制远近（越小越远）
    twinkle_phase: f32, // 用于轻微闪烁
}

struct Meteor {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    life: f32, // 生命值，逐渐减少
    max_life: f32,
}

fn new_star(rng: &mut impl Rng) -> Star {
    Star {
        x: rng.gen_range(STAR_SPAWN_MARGIN..WIDTH as f32 - STAR_SPAWN_MARGIN),
        y: rng.gen_range(STAR_SPAWN_MARGIN..HEIGHT as f32 - STAR_SPAWN_MARGIN),
        z: rng.gen_range(1.2..6.0), // 大部分较远
        twinkle_phase: rng.gen_range(0.0..std::f32::consts::TAU),
    }
}

fn new_meteor(rng: &mut impl Rng) -> Meteor {
    let side = rng.gen_range(0..3);
    let (x, y) = match side {
        0 => (-20.0, rng.gen_range(0.0..HEIGHT as f32 * 0.85)),
        1 => (rng.gen_range(0.0..WIDTH as f32), -20.0),
        _ => (
            WIDTH as f32 + 20.0,
            rng.gen_range(0.0..HEIGHT as f32 * 0.85),
        ),
    };
    let target_x = rng.gen_range(WIDTH as f32 * 0.15..WIDTH as f32 * 0.85);
    let target_y = rng.gen_range(HEIGHT as f32 * 0.15..HEIGHT as f32 * 0.85);
    let angle = (target_y - y).atan2(target_x - x);
    let speed = rng.gen_range(8.0..16.0);
    let life = rng.gen_range(70.0..130.0);

    Meteor {
        x,
        y,
        dx: speed * angle.cos(),
        dy: speed * angle.sin(),
        life,
        max_life: life,
    }
}

fn star_is_visible(star: &Star) -> bool {
    star.x.is_finite()
        && star.y.is_finite()
        && star.x >= 0.0
        && star.x < WIDTH as f32
        && star.y >= 0.0
        && star.y < HEIGHT as f32
}

fn update_star(star: &mut Star, rng: &mut impl Rng) {
    let center_x = WIDTH as f32 / 2.0;
    let center_y = HEIGHT as f32 / 2.0;
    let drift = STAR_DRIFT * (6.0 - star.z).max(0.0);

    star.x += (star.x - center_x) * drift;
    star.y += (star.y - center_y) * drift;

    if !star_is_visible(star) {
        *star = new_star(rng);
        return;
    }

    star.twinkle_phase = (star.twinkle_phase + TWINKLE_SPEED).rem_euclid(std::f32::consts::TAU);
}

fn put_pixel(buffer: &mut [u32], x: isize, y: isize, color: u32) {
    if x >= 0 && x < WIDTH as isize && y >= 0 && y < HEIGHT as isize {
        buffer[y as usize * WIDTH + x as usize] = color;
    }
}

fn draw_star(buffer: &mut [u32], star: &Star) -> bool {
    let brightness = (120.0 + (star.twinkle_phase.sin() * 50.0)) as u32;
    let size = if star.z > 4.0 { 0.8 } else { 1.2 };

    let sx = star.x as isize;
    let sy = star.y as isize;

    if sx < 0 || sx >= WIDTH as isize || sy < 0 || sy >= HEIGHT as isize {
        return false;
    }

    let color = (brightness << 16) | (brightness << 8) | brightness;
    put_pixel(buffer, sx, sy, color);

    // 轻微光晕（只给较亮的星星）
    if brightness > 140 && size > 1.0 {
        put_pixel(buffer, sx - 1, sy, color / 3);
        put_pixel(buffer, sx + 1, sy, color / 3);
    }

    true
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
    let mut stars: Vec<Star> = (0..STAR_COUNT).map(|_| new_star(&mut rng)).collect();

    let mut meteors: Vec<Meteor> = Vec::new();
    let mut frames_since_meteor = METEOR_FORCE_AFTER_FRAMES;

    window.set_target_fps(60); // ~60fps

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0);

        // 1. 绘制普通星星（稀疏 + 轻微闪烁）
        let mut drawn_stars = 0;
        for star in stars.iter_mut() {
            update_star(star, &mut rng);
            if draw_star(&mut buffer, star) {
                drawn_stars += 1;
            }
        }

        // 极端情况下如果整批星星状态异常，立即重建，避免出现纯黑帧。
        if drawn_stars == 0 {
            for star in stars.iter_mut() {
                *star = new_star(&mut rng);
                draw_star(&mut buffer, star);
            }
        }

        // 2. 随机产生流星（少量）
        frames_since_meteor += 1;
        if rng.gen::<f32>() < METEOR_CHANCE || frames_since_meteor >= METEOR_FORCE_AFTER_FRAMES {
            meteors.push(new_meteor(&mut rng));
            frames_since_meteor = 0;
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
                if i as f32 > m.life {
                    break;
                }
                let factor = 1.0 - (i as f32 / tail_steps as f32);
                let bright = (alpha as f32 * factor) as u32;

                let tx = (m.x - m.dx * i as f32 * 0.6) as isize;
                let ty = (m.y - m.dy * i as f32 * 0.6) as isize;

                put_pixel(
                    &mut buffer,
                    tx,
                    ty,
                    (bright << 16) | (bright << 8) | (bright / 2 + 40),
                ); // 偏白带点蓝
            }

            true
        });

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn stars_remain_drawable_after_long_run() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut stars: Vec<Star> = (0..STAR_COUNT).map(|_| new_star(&mut rng)).collect();

        for _ in 0..500_000 {
            for star in stars.iter_mut() {
                update_star(star, &mut rng);
                assert!(star.x.is_finite());
                assert!(star.y.is_finite());
                assert!(star.x >= 0.0 && star.x < WIDTH as f32);
                assert!(star.y >= 0.0 && star.y < HEIGHT as f32);
            }
        }

        let drawable = stars
            .iter()
            .filter(|star| {
                let sx = star.x as isize;
                let sy = star.y as isize;
                sx >= 0 && sx < WIDTH as isize && sy >= 0 && sy < HEIGHT as isize
            })
            .count();

        assert_eq!(drawable, STAR_COUNT);
    }

    #[test]
    fn stars_do_not_pile_up_on_edges() {
        let mut rng = StdRng::seed_from_u64(2);
        let mut stars: Vec<Star> = (0..STAR_COUNT).map(|_| new_star(&mut rng)).collect();

        for _ in 0..120_000 {
            for star in stars.iter_mut() {
                update_star(star, &mut rng);
            }
        }

        let edge_stars = stars
            .iter()
            .filter(|star| {
                star.x < 2.0
                    || star.x > WIDTH as f32 - 2.0
                    || star.y < 2.0
                    || star.y > HEIGHT as f32 - 2.0
            })
            .count();

        assert!(edge_stars < STAR_COUNT / 8);
    }

    #[test]
    fn meteors_are_aimed_toward_the_screen() {
        let mut rng = StdRng::seed_from_u64(3);

        for _ in 0..1_000 {
            let meteor = new_meteor(&mut rng);
            let center_x = WIDTH as f32 / 2.0;
            let center_y = HEIGHT as f32 / 2.0;
            let to_center_x = center_x - meteor.x;
            let to_center_y = center_y - meteor.y;
            let moving_toward_screen = meteor.dx * to_center_x + meteor.dy * to_center_y > 0.0;

            assert!(moving_toward_screen);
        }
    }
}
