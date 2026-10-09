use crate::scene::{Scene, HEIGHT, WIDTH};
use minifb::{Key, Window, WindowOptions};

pub fn run() -> Result<(), minifb::Error> {
    let mut window = Window::new(
        "Starfield - ESC to quit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;
    window.set_target_fps(60);

    let mut rng = rand::thread_rng();
    let mut scene = Scene::new(&mut rng);
    let mut buffer = vec![0; WIDTH * HEIGHT];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        scene.tick(&mut rng, &mut buffer);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;
    }
    Ok(())
}
