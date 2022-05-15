use solstice_2d::Draw;

fn main() {
    let (width, height) = (1024, 768);

    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("raytracer-viz")
        .with_inner_size(glutin::dpi::LogicalSize::new(width as f32, height as f32));
    let windowed_context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let glow = unsafe {
        solstice_2d::solstice::glow::Context::from_loader_function(|addr| {
            windowed_context.get_proc_address(addr)
        })
    };
    let mut gl = solstice_2d::solstice::Context::new(glow);
    let mut gfx = solstice_2d::Graphics::new(&mut gl, width as _, height as _).unwrap();

    let mut image = None;
    let (image_sx, image_rx) = std::sync::mpsc::sync_channel(1);
    let (notify_sx, notify_rx) = std::sync::mpsc::sync_channel(1);
    notify_sx.send(()).unwrap();
    std::thread::spawn(move || {
        while let Ok(_) = notify_rx.recv() {
            image_sx.send(file_rayz(width as _, height as _)).unwrap();
        }
    });

    el.run(move |event, _, cf| {
        use glutin::{event::*, event_loop::*};
        *cf = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *cf = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::R),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                let _r = notify_sx.try_send(());
            }
            Event::MainEventsCleared => {
                windowed_context.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                if let Ok(Ok(result)) = image_rx.try_recv() {
                    image = Some(
                        solstice_2d::solstice::image::Image::with_data(
                            &mut gl,
                            solstice_2d::solstice::texture::TextureType::Tex2D,
                            solstice_2d::solstice::PixelFormat::RGB8,
                            width as _,
                            height as _,
                            &result,
                            solstice_2d::solstice::image::Settings::default(),
                        )
                        .unwrap(),
                    );
                }

                {
                    let mut g = gfx.lock(&mut gl);
                    g.clear([1., 0., 0., 1.]);
                    if let Some(image) = &image {
                        g.image(
                            solstice_2d::Rectangle {
                                x: 0.0,
                                y: 0.0,
                                width: width as _,
                                height: height as _,
                            },
                            image,
                        );
                    }
                }
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}

fn file_rayz(
    width: usize,
    height: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    use glam::vec3a as vec3;
    use raytracer::*;

    let source = std::fs::read_to_string("src.ies")?;
    let parser = eisenscript::Parser::new(eisenscript::Lexer::new(&source));
    let rules = parser.rules().map_err(|err| format!("{}", err))?;

    let default_material = Material::Lambertian(Lambertian {
        color: vec3(1., 1., 1.),
        diffuse_weight: 0.8,
        ambient_weight: 0.2,
        texture: None,
    });

    let mut rng = rand::thread_rng();
    let models = rules
        .iter(&mut eisenscript::ContextMut::new(&mut rng))
        .map(|(tx, p)| {
            use eisenscript::Primitive as EP;
            let sdf = match p {
                EP::Box => Primitive::Box {
                    size: vec3(0.5, 0.5, 0.5),
                },
                EP::Sphere => Primitive::Sphere { radius: 0.5 },
                _ => unimplemented!(),
            };
            let transform: mint::ColumnMatrix4<f32> = tx.into();
            println!("{:?}", transform);
            Model {
                transform: transform.into(),
                sdf,
                material: default_material.clone(),
            }
        })
        .collect();
    let scene = Scene {
        width,
        height,
        camera: Camera {
            transform: glam::Affine3A::look_at_lh(
                glam::vec3(0., 0., -1.),
                glam::vec3(0., 0., 0.),
                glam::vec3(0., -1., 0.),
            ),
        },
        models,
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

    Ok(image_data)
}

// fn rayz(width: usize, height: usize) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
//     use glam::{vec3a as vec3, Affine3A as Transform};
//     use raytracer::*;
//
//     let default_mat = Lambertian {
//         color: vec3(1., 1., 1.),
//         diffuse_weight: 0.8,
//         ambient_weight: 0.2,
//         texture: None,
//     };
//     let scene = Scene {
//         width,
//         height,
//         camera: Camera {
//             transform: glam::Affine3A::from_rotation_translation(
//                 glam::Quat::from_rotation_y(0.),
//                 glam::vec3(0., 0., -1.),
//             ),
//         },
//         models: vec![
//             Model {
//                 transform: Transform::from_translation(glam::vec3(-1., 0., 1.)),
//                 sdf: Primitive::Sphere { radius: 0.5 },
//                 material: Material::Lambertian(Lambertian {
//                     ..default_mat.clone()
//                 }),
//             },
//             Model {
//                 transform: Transform::from_rotation_translation(
//                     glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_3),
//                     glam::vec3(1., 0., 1.),
//                 ),
//                 sdf: Primitive::Box {
//                     size: vec3(1., 1., 1.) * 0.5,
//                 },
//                 material: Material::Lambertian(default_mat.clone()),
//             },
//         ],
//         lights: vec![Light {
//             color: vec3(1., 1., 1.),
//             intensity: 1.0,
//             position: vec3(0., 0., 1.),
//         }],
//     };
//
//     let start = std::time::Instant::now();
//     let image_data = scene.render();
//     let dt = start.elapsed();
//     println!("dt: {:?}", dt);
//
//     Ok(image_data)
// }
