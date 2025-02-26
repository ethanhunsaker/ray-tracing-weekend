mod vec3;
mod color;
mod ray;

use std::io;
use std::ops::Mul;
use crate::color::Color;
use crate::ray::Ray;

fn ray_color(r: &Ray) -> Color {
    let unit_direction = vec3::unit_vector(r.direction());
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn main() {
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: i32 = 256;
    const IMAGE_HEIGHT: i32 = 256;

    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    for j in 0..IMAGE_HEIGHT {
        eprint!("\rScanlines remaining: {}", (IMAGE_HEIGHT - j));
        for i in 0..IMAGE_WIDTH {
            let r = i as f64 / (IMAGE_WIDTH - 1) as f64;
            let g = j as f64 / (IMAGE_HEIGHT - 1) as f64;
            let b = 0.00;
            let pixel_color = Color::new(r,g,b);
            color::write_color(&mut io::stdout(), pixel_color);
        }
    }

    eprint!("\nDone.\n");
}
