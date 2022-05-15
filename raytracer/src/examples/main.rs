use glam::{vec3a as vec3, Affine3A as Transform};
use spooky_raytracer::*;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let default_mat = Lambertian {
        color: vec3(1., 1., 1.),
        diffuse_weight: 0.8,
        ambient_weight: 0.2,
    };
    let (height, width) = ((256. * 16. / 9.) as _, 256);
    let scene = Scene {
        width,
        height,
        camera: Camera {
            transform: glam::Affine3A::from_rotation_translation(
                glam::Quat::from_rotation_y(0.),
                glam::vec3(0., 0., -1.),
            ),
        },
        models: vec![
            Model {
                transform: Transform::from_translation(glam::vec3(-1., 0., 1.)),
                sdf: Primitive::Sphere { radius: 0.5 },
                material: Material::Lambertian(Lambertian {
                    ..default_mat.clone()
                }),
            },
            Model {
                transform: Transform::from_rotation_translation(
                    glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_3),
                    glam::vec3(1., 0., 1.),
                ),
                sdf: Primitive::Box {
                    size: vec3(1., 1., 1.) * 0.5,
                },
                material: Material::Lambertian(default_mat.clone()),
            },
            Model {
                transform: Transform::from_rotation_translation(
                    glam::Quat::from_rotation_x(std::f32::consts::FRAC_PI_3)
                        * glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
                    glam::vec3(0., 1., 1.),
                ),
                sdf: Primitive::Dynamic({
                    let b = Primitive::Box {
                        size: vec3(1., 1., 1.) * 0.25,
                    };
                    let radius = 0.1;
                    Box::new(move |p| b.eval(p) - radius)
                }),
                material: Material::Lambertian(default_mat.clone()),
            },
        ],
        lights: vec![Light {
            color: vec3(1., 1., 1.),
            intensity: 1.0,
            position: vec3(0., 0., 1.),
        }],
    };

    let start = std::time::Instant::now();
    let image_data = scene.render();
    let dt = start.elapsed();
    println!("dt: {:?}", dt);

    let image = image::RgbImage::from_raw(width as _, height as _, image_data).unwrap();
    image.save("out.png")?;

    Ok(())
}
