mod camera;
mod color;
mod common;
mod hittable;
mod hittable_list;
mod material;
mod ray;
mod sphere;
mod vec3;

use std::fs;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use rayon::prelude::*;

use crate::camera::Camera;
use crate::color::Color;
use crate::hittable::Hittable;
use crate::hittable_list::HittableList;
use crate::material::{Dielectric, Lambertian, Metal};
use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::vec3::Point3;

fn ray_color(r: &Ray, world: &dyn Hittable, depth: i32) -> Color {
    // If we've exceeded the ray bounce limit, no more light is gathered
    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(hit_rec) = world.hit(r, 0.001, common::INFINITY) {
        if let Some(scatter_rec) = hit_rec.mat.scatter(r, &hit_rec) {
            return scatter_rec.attenuation * ray_color(&scatter_rec.scattered, world, depth - 1);
        }
        return Color::new(0.0, 0.0, 0.0);
    }

    let unit_direction = vec3::unit_vector(r.direction());
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn random_scene() -> HittableList {
    let mut world = HittableList::new();

    let ground_material = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = common::random_double();
            let center = Point3::new(
                a as f64 + 0.9 * common::random_double(),
                0.2,
                b as f64 + 0.9 * common::random_double(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // Diffuse
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Arc::new(Lambertian::new(albedo));
                    world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
                } else if choose_mat < 0.95 {
                    // Metal
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = common::random_double_range(0.5, 1.0);
                    let sphere_material = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
                } else {
                    // Glass
                    let sphere_material = Arc::new(Dielectric::new(1.5));
                    world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
                }
            }
        }
    }

    let material1 = Arc::new(Dielectric::new(1.5));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1,
    )));

    let material2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material2,
    )));

    let material3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material3,
    )));

    world
}

fn main() {
    const ASPECT_RATIO: f64 = 3.0 / 2.0;
    const IMAGE_WIDTH: i32 = 1200;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
    const SAMPLES_PER_PIXEL: i32 = 500;
    const MAX_DEPTH: i32 = 50;

    // World

    let world = random_scene();

    for frame in 0..180 {
        eprintln!("\rFrame {} started", frame);

        let angle_per_frame = 2.0 * std::f64::consts::PI / 180.0;
        let angle = frame as f64 * angle_per_frame;

        let radius = 13.0;
        let height = 2.0;

        let x = radius * angle.cos();
        let z = radius * angle.sin();

        // Camera

        let lookfrom = Point3::new(x, height, z);
        let lookat = Point3::new(0.0, 0.0, 0.0);
        let vup = Point3::new(0.0, 1.0, 0.0);
        let dist_to_focus = 10.0;
        let aperture = 0.1;

        let cam = Camera::new(
            lookfrom,
            lookat,
            vup,
            20.0,
            ASPECT_RATIO,
            aperture,
            dist_to_focus,
        );

        // Render

        fs::create_dir_all("out").expect("Failed to create output directory");

        let filename = format!("out/frame_{:03}.ppm", frame);
        let file =
            fs::File::create(&filename).expect(&format!("Failed to create file: {}", filename));
        let mut writer = BufWriter::new(file);

        writeln!(&mut writer, "P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT)
            .expect("Error writing header");

        for j in (0..IMAGE_HEIGHT).rev() {
            eprint!("\rFrame: {}, Scanlines remaining: {}", frame, j);
            let pixel_colors: Vec<_> = (0..IMAGE_WIDTH)
                .into_par_iter()
                .map(|i| {
                    let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                    for _ in 0..SAMPLES_PER_PIXEL {
                        let u = ((i as f64) + common::random_double()) / (IMAGE_WIDTH - 1) as f64;
                        let v = ((j as f64) + common::random_double()) / (IMAGE_HEIGHT - 1) as f64;
                        let r = cam.get_ray(u, v);
                        pixel_color += ray_color(&r, &world, MAX_DEPTH);
                    }
                    pixel_color
                })
                .collect();
            for pixel_color in pixel_colors {
                color::write_color(&mut writer, pixel_color, SAMPLES_PER_PIXEL);
            }
        }

        eprintln!("\rFrame {} completed", frame);
    }
}
