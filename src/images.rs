use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub async fn rayz(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    let buffer = tokio::task::spawn_blocking::<
        _,
        std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>,
    >(move || {
        use raytracer::glam::vec3a as vec3;
        use raytracer::*;
        let default_mat = Lambertian {
            color: vec3(1., 1., 1.),
            diffuse_weight: 0.8,
            ambient_weight: 0.2,
            texture: None,
        };
        let (width, height) = (1280, 720);
        let scene = Scene {
            width,
            height,
            camera: Camera {
                transform: Default::default(),
            },
            models: vec![Model {
                transform: Default::default(),
                sdf: Primitive::Sphere { radius: 0.5 },
                material: Material::Lambertian(default_mat.clone()),
            }],
            lights: vec![Light {
                color: vec3(1., 1., 1.),
                intensity: 1.0,
                position: vec3(1., 0., 1.),
            }],
        };
        let pixels = scene.render();

        let mut buffer = vec![];
        let mut encoder = png::Encoder::new(&mut buffer, width as _, height as _);
        encoder.set_color(png::ColorType::Rgb);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&pixels)?;
        writer.finish()?;
        Ok(buffer)
    })
    .await?
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
