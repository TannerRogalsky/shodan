use serenity::builder::CreateApplicationCommands;
use serenity::model::prelude::application_command::{
    ApplicationCommand, ApplicationCommandInteraction,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

// mod grim;
mod images;
mod jeopardy;

struct Handler;

const JEOPARDY_CMD: &'static str = "jeopardy";
const RAYZ_CMD: &'static str = "rayz";

#[serenity::async_trait]
impl EventHandler for Handler {
    /// Dispatched upon startup.
    ///
    /// Provides data about the bot and the guilds it's in.
    async fn ready(&self, ctx: Context, data: Ready) {
        println!("BOT READY");

        fn create_interactions(
            commands: &mut CreateApplicationCommands,
        ) -> &mut CreateApplicationCommands {
            commands
                .create_application_command(|command| {
                    command
                        .name(JEOPARDY_CMD)
                        .description("Displays a random jeopardy category.")
                })
                .create_application_command(|command| {
                    command
                        .name(RAYZ_CMD)
                        .description("Ray traces a random image.")
                })
        }

        for guild in data.guilds {
            let r = guild.id.set_application_commands(&ctx.http, |c| c).await;
            if let Err(err) = r {
                let guild = Guild::get(&ctx.http, guild.id).await;
                let name = guild
                    .as_ref()
                    .map(|guild| guild.name.as_str())
                    .unwrap_or("unknown");
                println!("Can't set application commands for {}: {}", name, err);
            }
        }

        ApplicationCommand::set_global_application_commands(&ctx.http, create_interactions)
            .await
            .unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let result = match command.data.name.as_str() {
                JEOPARDY_CMD => jeopardy(&ctx, command).await,
                RAYZ_CMD => rayz(&ctx, command).await,
                _ => command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content("not implemented :(")
                            })
                    })
                    .await
                    .map_err(Into::into),
            };
            if let Err(why) = result {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    // Login with a bot token from the environment
    let token = std::env::var("BOT_TOKEN").expect("token");
    assert!(serenity::utils::validate_token(&token).is_ok());

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .type_map_insert::<jeopardy::Jeopardy>(jeopardy::Jeopardy::new().unwrap())
        .await
        .expect("Error creating client");

    // {
    //     // Open the data lock in write mode, so keys can be inserted to it.
    //     let mut data = client.data.write().await;
    //
    //     data.insert::<grim::GrimGames>(Default::default());
    //     data.insert::<grim::GrimBuilders>(Default::default());
    // }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

// async fn grim(ctx: &Context, msg: &Message) -> serenity::framework::standard::CommandResult {
//     use structopt::StructOpt;
//     let args = msg
//         .content
//         .split_ascii_whitespace()
//         .map(Into::<std::ffi::OsString>::into);
//     match grim::GrimCmd::from_iter_safe(args) {
//         Ok(cmd) => {
//             let ctx = grim::GrimContext::new(ctx, msg).await;
//             cmd.execute(ctx).await?;
//         }
//         Err(e) => {
//             msg.reply(ctx, e.message).await?;
//         }
//     }
//
//     Ok(())
// }

async fn jeopardy(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    let data = ctx.data.read().await;
    let content: std::borrow::Cow<'static, str> = match data.get::<jeopardy::Jeopardy>() {
        Some(jeopardy) => match jeopardy.random() {
            Ok(category) => jeopardy::Jeopardy::fmt_category(&category).into(),
            Err(err) => err.into(),
        },
        None => "Jeopardy module not loaded.".into(),
    };

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await?;
    Ok(())
}

async fn rayz(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    images::rayz(ctx, command).await
}

#[cfg(test)]
mod tests {}
