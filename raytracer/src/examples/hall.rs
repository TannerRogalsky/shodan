use glam::{vec3a as vec3, Affine3A as Transform};
use spooky_raytracer::*;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let size = 256.;
    let (width, height) = ((size * 16. / 9.) as _, size as _);
    let default_mat = Material::Lambertian(Lambertian {
        color: vec3(1., 1., 1.),
        diffuse_weight: 0.8,
        ambient_weight: 0.2,
    });
    let camera = Camera {
        transform: glam::Affine3A::from_rotation_translation(
            glam::Quat::from_rotation_y(0.),
            glam::vec3(0., 0., -2.5),
        ),
    };
    let hall = Primitive::Dynamic({
        let floor = Primitive::Box {
            size: vec3(1., 0.1, 1.) * 0.5,
        };
        let wall_z = Primitive::Box {
            size: vec3(0.1, 1.0, 1.) * 0.5,
        };
        Box::new(move |p: glam::Vec3A| {
            let floor = floor.eval(p);
            let horizontal_offset = vec3(0.5, 0., 0.);
            let vertical_offset = vec3(0., -0.5, 0.);
            let left = wall_z.eval(p - horizontal_offset + vertical_offset);
            let right = wall_z.eval(p + horizontal_offset + vertical_offset);
            floor.min(left).min(right)
        })
    });
    let origin_light = camera.transform.transform_vector3a(vec3(0., 0., 0.));
    let scene = Scene {
        width,
        height,
        camera,
        models: vec![Model {
            transform: Transform::from_translation(glam::vec3(0., -0.5, 0.)),
            sdf: hall,
            material: default_mat.clone(),
        }],
        lights: vec![Light {
            color: vec3(0.1, 0.6, 1.0),
            intensity: 1.0,
            position: origin_light,
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
