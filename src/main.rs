use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

mod grim;
mod images;
mod jeopardy;

#[group]
#[commands(grim, jeopardy, earth)]
struct General;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    /// Dispatched upon startup.
    ///
    /// Provides data about the bot and the guilds it's in.
    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        println!("BOT READY");
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = std::env::var("BOT_TOKEN").expect("token");
    assert!(serenity::utils::validate_token(&token).is_ok());

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<jeopardy::Jeopardy>(jeopardy::Jeopardy::new().unwrap())
        .await
        .expect("Error creating client");

    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;

        data.insert::<grim::GrimGames>(Default::default());
        data.insert::<grim::GrimBuilders>(Default::default());
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn grim(ctx: &Context, msg: &Message) -> CommandResult {
    use structopt::StructOpt;
    let args = msg
        .content
        .split_ascii_whitespace()
        .map(Into::<std::ffi::OsString>::into);
    match grim::GrimCmd::from_iter_safe(args) {
        Ok(cmd) => {
            let ctx = grim::GrimContext::new(ctx, msg).await;
            cmd.execute(ctx).await?;
        }
        Err(e) => {
            msg.reply(ctx, e.message).await?;
        }
    }

    Ok(())
}

#[command]
async fn jeopardy(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    match data.get::<jeopardy::Jeopardy>() {
        Some(jeopardy) => match jeopardy.random() {
            Ok(category) => {
                msg.reply(ctx, jeopardy::Jeopardy::fmt_category(&category))
                    .await?;
            }
            Err(err) => {
                msg.reply(ctx, err).await?;
            }
        },
        None => {
            msg.reply(ctx, "Jeopardy module not loaded.").await?;
        }
    }

    Ok(())
}

#[command]
async fn earth(ctx: &Context, msg: &Message) -> CommandResult {
    let rot = msg
        .content
        .split_ascii_whitespace()
        .last()
        .and_then(|rot| rot.parse::<f64>().ok())
        .unwrap_or(0.);
    println!("ROT: {}", rot);
    let rot = rot / 360. % 1.;
    images::earth(ctx, msg.channel_id, rot).await
}

#[cfg(test)]
mod tests {}
