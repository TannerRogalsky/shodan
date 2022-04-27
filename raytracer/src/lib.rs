use raytracer::point3d::Point3D;
use raytracer::raytracer::render_to_buffer;

pub struct Pixels {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

pub fn run(rot: f64) -> Pixels {
    let image_width = 1280;
    let image_height = 720;

    let scene = raytracer::config::Config {
        width: image_width,
        height: image_height,
        samples_per_pixel: 128,
        max_depth: 50,
        sky: Some(raytracer::config::Sky::new_default_sky()),
        camera: raytracer::camera::CameraParams {
            look_from: Point3D::new(0., 0., 1.),
            look_at: Point3D::new(0., 0., -1.),
            vup: Point3D::new(0., 1.0, 0.),
            vfov: 90.0,
            aspect: 1280. / 720.,
        }
        .into(),
        objects: vec![raytracer::sphere::Sphere {
            center: Point3D::new(0., 0., 0.),
            radius: 0.5,
            material: raytracer::materials::Material::Texture(raytracer::materials::Texture::new(
                (1., 1., 1.).into(),
                "data/2k_earth_daymap.jpg",
                rot,
            )),
        }],
    };

    let pixels = render_to_buffer(scene);
    Pixels {
        width: image_width,
        height: image_height,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
