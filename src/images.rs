use glam::vec3a as vec3;
use raytracer::*;
use serenity::model::application::interaction::{
    application_command::ApplicationCommandInteraction, InteractionResponseType,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

pub async fn rayz(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    let source = command
        .data
        .options
        .get(0)
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap();

    let buffer = tokio::task::block_in_place::<
        _,
        std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>,
    >(move || {
        let parser = eisenscript::Parser::new(eisenscript::Lexer::new(source));
        let rules = parser.rules().map_err(|err| format!("{}", err))?;

        let mut rng = rand::thread_rng();
        let models = rules
            .iter(&mut eisenscript::ContextMut::new(&mut rng))
            .map(eis_to_model)
            .collect();
        let scene = Scene {
            width: WIDTH as _,
            height: HEIGHT as _,
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
        let pixels = scene.render();
        let dt = start.elapsed();
        println!("dt: {:?}", dt);

        let mut buffer = vec![];
        let mut encoder = png::Encoder::new(&mut buffer, WIDTH as _, HEIGHT as _);
        encoder.set_color(png::ColorType::Rgb);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&pixels)?;
        writer.finish()?;
        Ok(buffer)
    })
    .map_err(|err| eyre::eyre!("{}", err))?;

    command
        .create_interaction_response(&ctx.http, move |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.add_file(AttachmentType::Bytes {
                        data: buffer.into(),
                        filename: "rayz.png".to_string(),
                    })
                })
        })
        .await?;
    Ok(())
}

fn eis_to_model((tx, p): (eisenscript::Transform, eisenscript::Primitive)) -> Model {
    let default_material = Material::Lambertian(Lambertian {
        color: vec3(1., 1., 1.),
        diffuse_weight: 0.8,
        ambient_weight: 0.2,
        texture: None,
    });

    use eisenscript::Primitive as EP;
    let sdf = match p {
        EP::Box => Primitive::Box {
            size: glam::vec3a(0.5, 0.5, 0.5),
        },
        EP::Sphere => Primitive::Sphere { radius: 0.5 },
        _ => unimplemented!(),
    };
    let transform: mint::ColumnMatrix4<f32> = tx.into();
    Model {
        transform: transform.into(),
        sdf,
        material: default_material.clone(),
    }
}
