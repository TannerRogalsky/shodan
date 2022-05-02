use serenity::model::interactions::application_command::{
    ApplicationCommandInteractionDataOption, ApplicationCommandOptionType,
};
use serenity::model::prelude::application_command::{
    ApplicationCommand, ApplicationCommandInteraction,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

// mod grim;
mod images;
mod jeopardy;
mod spirits_awaken;
mod wumpus;

struct Handler;

const JEOPARDY_CMD: &'static str = "jeopardy";
const RAYZ_CMD: &'static str = "rayz";
const WUMPUS_CMD: &'static str = "htw";
const SPIRITS_CMD: &'static str = "spirits";

#[serenity::async_trait]
impl EventHandler for Handler {
    /// Dispatched upon startup.
    ///
    /// Provides data about the bot and the guilds it's in.
    async fn ready(&self, ctx: Context, data: Ready) {
        println!("BOT READY");

        fn create_interactions(
            commands: &mut serenity::builder::CreateApplicationCommands,
        ) -> &mut serenity::builder::CreateApplicationCommands {
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
                .create_application_command(|command| {
                    command
                        .name(WUMPUS_CMD)
                        .description("Plays Hunt The Wumpus")
                        .create_option(|option| {
                            option
                                .name("start")
                                .description("Start a new game of Hunt The Wumpus")
                                .kind(ApplicationCommandOptionType::SubCommand)
                        })
                        .create_option(|option| {
                            option
                                .name("status")
                                .description(
                                    "Shows the current status of your Hunt The Wumpus game.",
                                )
                                .kind(ApplicationCommandOptionType::SubCommand)
                        })
                        .create_option(|option| {
                            option
                                .name("move")
                                .description("Move to a new room.")
                                .kind(ApplicationCommandOptionType::SubCommand)
                                .create_sub_option(|option| {
                                    option
                                        .name("room")
                                        .description("The room to move to.")
                                        .kind(ApplicationCommandOptionType::Integer)
                                        .max_int_value(19)
                                        .required(true)
                                })
                        })
                        .create_option(|option| {
                            option
                                .name("shoot")
                                .description("Shoot an arrow into an adjacent room.")
                                .kind(ApplicationCommandOptionType::SubCommand)
                                .create_sub_option(|option| {
                                    option
                                        .name("at_room")
                                        .description("The room to shoot into.")
                                        .kind(ApplicationCommandOptionType::Integer)
                                        .max_int_value(19)
                                        .required(true)
                                })
                                .create_sub_option(|option| {
                                    option
                                        .name("room_travel")
                                        .description("The number of rooms for the arrow to travel.")
                                        .kind(ApplicationCommandOptionType::Integer)
                                        .max_int_value(5)
                                        .required(true)
                                })
                        })
                })
                .create_application_command(|commands| {
                    commands
                        .name(SPIRITS_CMD)
                        .description("Version 0.0035")
                        .create_option(|option| {
                            option
                                .name("generate")
                                .description("Generate a new character.")
                                .kind(ApplicationCommandOptionType::SubCommand)
                        })
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
                WUMPUS_CMD => htw(&ctx, command).await,
                SPIRITS_CMD => spirits(&ctx, command).await,
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
        .type_map_insert::<HTWGamesTypeMap>(std::default::Default::default())
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

async fn spirits(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    let sub = command.data.options.get(0);
    let content: std::borrow::Cow<'static, str> = match sub.as_ref().map(|sub| sub.name.as_str()) {
        Some("generate") => spirits_awaken::generate().format().into(),
        _ => "Unrecognized subcommand.".into(),
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

type HTWGames = std::collections::HashMap<UserId, wumpus::HuntTheWumpus>;
#[derive(Default)]
struct HTWGamesTypeMap(std::sync::Arc<tokio::sync::Mutex<HTWGames>>);
impl TypeMapKey for HTWGamesTypeMap {
    type Value = Self;
}

async fn htw(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    use std::collections::hash_map::Entry;

    let data = ctx.data.read().await;
    let games_ty = data.get::<HTWGamesTypeMap>().unwrap();
    let mut games = games_ty.0.lock().await;
    let maybe_game = games.entry(command.user.id);

    let cmd = &command.data.options[0];
    let content: std::borrow::Cow<'static, str> = match cmd.name.as_str() {
        "start" => match maybe_game {
            Entry::Occupied(entry) => format!(
                "There is already a game running for you. \n{}",
                entry.get().status()
            )
            .into(),
            Entry::Vacant(entry) => {
                let game = wumpus::HuntTheWumpus::new();
                let status = game.status();
                entry.insert(game);
                status.into()
            }
        },
        name => match maybe_game {
            Entry::Occupied(mut game) => {
                let content = match name {
                    "status" => game.get().status().into(),
                    "move" => {
                        let new_room =
                            cmd.options[0].value.as_ref().unwrap().as_u64().unwrap() as usize;
                        if game.get_mut().move_player_to(new_room) {
                            game.get().status().into()
                        } else {
                            format!(
                                "That's not a room you can move to!\n{}",
                                game.get().status()
                            )
                            .into()
                        }
                    }
                    "shoot" => {
                        fn get_usize(
                            name: &'static str,
                        ) -> impl FnMut(&ApplicationCommandInteractionDataOption) -> Option<usize>
                        {
                            move |option| {
                                (option.name == name).then(|| {
                                    option.value.as_ref().unwrap().as_u64().unwrap() as usize
                                })
                            }
                        }

                        let at_room = cmd.options.iter().find_map(get_usize("at_room")).unwrap();
                        let room_travel = cmd
                            .options
                            .iter()
                            .find_map(get_usize("room_travel"))
                            .unwrap();
                        if game.get_mut().shoot(at_room, room_travel) {
                            game.get().status().into()
                        } else {
                            format!(
                                "That's not a room you can shoot into!\n{}",
                                game.get().status()
                            )
                            .into()
                        }
                    }
                    _ => "Unrecognized subcommand".into(),
                };
                if !game.get().is_running() {
                    game.remove_entry();
                }
                content
            }
            Entry::Vacant(_) => "There's no game running for you.".into(),
        },
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
