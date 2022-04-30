use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub async fn earth(ctx: &Context, channel: ChannelId, rot: f64) -> CommandResult {
    let buffer = tokio::task::spawn_blocking::<
        _,
        std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>,
    >(move || {
        let pixels = raytracer::run(rot);

        let mut buffer = vec![];
        let mut encoder = png::Encoder::new(&mut buffer, pixels.width as _, pixels.height as _);
        encoder.set_color(png::ColorType::Rgb);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&pixels.pixels)?;
        writer.finish()?;
        Ok(buffer)
    })
    .await??;

    channel
        .send_message(&ctx.http, |m| {
            m.add_file(AttachmentType::Bytes {
                data: buffer.into(),
                filename: "earth.png".to_string(),
            })
        })
        .await?;
    Ok(())
}
