extern crate rand;

use rand::Rng;

pub struct Circle {
    radius: f32,
    center: [f32; 2],
    color: [f32; 4],
}

pub fn create_random_scene(screen_width: u32, screen_height: u32) -> Vec<Circle> {
    let mut rng = rand::thread_rng();

    let num_circles: u32 = 1000;

    let mut circles: Vec<Circle> = Vec::new();

    for n in 0..1000 {
        circles.push(Circle {
            radius: rng.gen_range(10.0, 100.0),
            center: [
                rng.gen_range(0.0, screen_width as f32),
                rng.gen_range(0.0, screen_height as f32),
            ],
            color: [rng.gen(), rng.gen(), rng.gen(), rng.gen()],
        })
    }

    circles
}
