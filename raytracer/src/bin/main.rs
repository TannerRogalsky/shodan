use spooky_raytracer::*;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let earth_texture_file = image::io::Reader::open("data/2k_earth_daymap.jpg")?;
    let earth_texture = earth_texture_file.decode()?;

    let default_mat = Lambertian {
        color: glam::vec3(1., 1., 1.),
        diffuse_weight: 0.8,
        ambient_weight: 0.2,
        texture: None,
    };
    let (width, height) = (1280, 720);
    let scene = Scene {
        width,
        height,
        camera: Camera {
            position: glam::vec3(0., 0., -1.),
        },
        models: vec![
            Model {
                position: glam::vec3(-1., 0., 1.),
                sdf: Primitive::Sphere { radius: 0.5 },
                material: Material::Lambertian(Lambertian {
                    texture: Some(earth_texture),
                    ..default_mat.clone()
                }),
            },
            Model {
                position: glam::vec3(1., 0., 1.),
                sdf: Primitive::Box {
                    size: glam::vec3(1., 1., 1.) * 0.5,
                },
                material: Material::Lambertian(default_mat.clone()),
            },
            Model {
                position: glam::vec3(0., 1., 1.),
                sdf: Primitive::Dynamic({
                    let b = Primitive::Box {
                        size: glam::vec3(1., 1., 1.) * 0.25,
                    };
                    let radius = 0.1;
                    Box::new(move |p: glam::Vec3| b.eval(p) - radius)
                }),
                material: Material::Lambertian(default_mat.clone()),
            },
        ],
        lights: vec![Light {
            color: glam::vec3(1., 1., 1.),
            intensity: 1.0,
            position: glam::vec3(0., 0., 1.),
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
