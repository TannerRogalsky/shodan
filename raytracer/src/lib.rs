pub use glam;
use glam::{vec3a as vec3, Vec3A as Vec3};

pub struct Scene {
    pub width: usize,
    pub height: usize,
    pub camera: Camera,
    pub models: Vec<Model>,
    pub lights: Vec<Light>,
}

impl Scene {
    fn pixel(&self, x: usize, y: usize) -> Vec3 {
        let w = self.width as f32;
        let h = self.height as f32;
        let mut p = self.camera.transform.transform_point3a(vec3(0., 0., 0.));
        let screen = vec3(x as _, y as _, 0.);
        let scale_factor = (1. / h) * -2.;
        let uv = (screen - vec3(w / 2., h / 2., 0.)) * scale_factor;
        let ray_dir = self
            .camera
            .transform
            .inverse()
            .transform_vector3a(uv - p)
            .normalize();

        const MAX_STEPS: usize = 64;
        const EPSILON: f32 = 0.01;
        for _ in 0..MAX_STEPS {
            let model = self
                .models
                .iter()
                .min_by(|a, b| a.distance_to(p).partial_cmp(&b.distance_to(p)).unwrap())
                .unwrap();
            let distance = model.distance_to(p);
            if distance < EPSILON {
                const H: f32 = 0.001; // approximate gradient with limit as h goes to zero (sufficiently small h)
                let normal = vec3(
                    (model.distance_to(p + vec3(H, 0., 0.)) - distance) / H,
                    (model.distance_to(p + vec3(0., H, 0.)) - distance) / H,
                    (model.distance_to(p + vec3(0., 0., H)) - distance) / H,
                )
                .normalize();

                let color = self
                    .lights
                    .iter()
                    .map(|light| {
                        model.shade(HitRecord {
                            light,
                            normal,
                            point: p,
                            model,
                        })
                    })
                    .fold(vec3(0., 0., 0.), |acc, color| acc + color);
                return color;
            }

            p = p + ray_dir * distance;
        }

        return vec3(1., 0., y as f32 / h); // bg color
    }

    pub fn render(&self) -> Vec<u8> {
        use bytemuck::{Pod, Zeroable};
        use rayon::prelude::*;

        let w = self.width;
        let h = self.height;

        #[derive(Copy, Clone, Pod, Zeroable)]
        #[repr(C)]
        struct Pixel {
            r: u8,
            g: u8,
            b: u8,
        }

        let mut buffer = vec![0; w * h * 3];
        let pixels = bytemuck::cast_slice_mut::<_, Pixel>(&mut buffer);
        pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
            let x = i % w;
            let y = i / w;
            let v = self.pixel(x, y);
            pixel.r = (v.x * 255.) as u8;
            pixel.g = (v.y * 255.) as u8;
            pixel.b = (v.z * 255.) as u8;
        });
        buffer
    }
}

pub struct Camera {
    pub transform: glam::Affine3A,
}

pub struct Model {
    pub transform: glam::Affine3A,
    pub sdf: Primitive,
    pub material: Material,
}

impl Model {
    fn distance_to(&self, p: Vec3) -> f32 {
        self.sdf.eval(self.transform.inverse().transform_point3a(p))
    }
    fn shade(&self, hit: HitRecord) -> Vec3 {
        self.material.shade(hit)
    }
}

pub enum Primitive {
    Sphere { radius: f32 },
    Box { size: Vec3 },
    Dynamic(Box<dyn SDF>),
}

impl Primitive {
    pub fn eval(&self, point: Vec3) -> f32 {
        match self {
            Primitive::Sphere { radius } => point.length() - radius,
            Primitive::Dynamic(sdf) => sdf(point),
            Primitive::Box { size } => {
                let q = point.abs() - *size;
                q.max(vec3(0., 0., 0.)).length() + q.x.max(q.y.max(q.z)).min(0.)
            }
        }
    }
}

pub trait SDF: Fn(Vec3) -> f32 + Send + Sync {}
impl<T> SDF for T where T: Fn(Vec3) -> f32 + Send + Sync {}

pub struct Light {
    pub color: Vec3,
    pub intensity: f32,
    pub position: Vec3,
}

#[derive(Clone)]
pub enum Material {
    Lambertian(Lambertian),
}

#[derive(Clone)]
pub struct Lambertian {
    pub color: Vec3,
    pub diffuse_weight: f32,
    pub ambient_weight: f32,
    pub texture: Option<image::DynamicImage>,
}

impl Material {
    fn shade(&self, hit: HitRecord) -> Vec3 {
        match self {
            Material::Lambertian(inner) => {
                let HitRecord {
                    light,
                    normal,
                    point,
                    model,
                    ..
                } = hit;
                let Lambertian {
                    color,
                    diffuse_weight,
                    ambient_weight,
                    ref texture,
                } = *inner;
                let light_dir = (light.position - point).normalize();
                let brightness = light_dir.dot(normal) * light.intensity;
                let light_shading_color = light.color * brightness;

                let texture_color = texture
                    .as_ref()
                    .map(|texture| {
                        let [u, v] = u_v_from_sphere_hit_point(point - model.transform.translation);
                        let v = 1. - v;
                        let x = ((texture.width() - 1) as f32 * u).floor() as u32;
                        let y = ((texture.height() - 1) as f32 * v).floor() as u32;
                        let [r, g, b, _a] = image::GenericImageView::get_pixel(texture, x, y).0;
                        vec3(r as f32 / 255., g as f32 / 255., b as f32 / 255.)
                    })
                    .unwrap_or(vec3(1., 1., 1.));

                let diffuse = (color * texture_color * light_shading_color).max(vec3(0., 0., 0.));
                let ambient = color * texture_color;
                diffuse * diffuse_weight + ambient * ambient_weight
            }
        }
    }
}

struct HitRecord<'a> {
    light: &'a Light,
    normal: Vec3,
    point: Vec3,
    model: &'a Model,
}

fn u_v_from_sphere_hit_point(hit_point_on_sphere: Vec3) -> [f32; 2] {
    let n = hit_point_on_sphere.normalize();
    let [x, y, z] = n.to_array();
    let u = (x.atan2(z) / (2.0 * std::f32::consts::PI)) + 0.5;
    let v = y * 0.5 + 0.5;
    [u, v]
}
