use raytracer::point3d::Point3D;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let scene = raytracer::config::Config {
        width: 1280,
        height: 720,
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
                0.,
            )),
        }],
    };
    raytracer::raytracer::render("test1.png", scene);

    Ok(())
}
