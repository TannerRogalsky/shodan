use crate::jeopardy::Vote;
use serenity::builder::CreateEmbed;
use serenity::model::interactions::application_command::{
    ApplicationCommandInteractionDataOption, ApplicationCommandOptionType,
};
use serenity::model::prelude::application_command::{
    ApplicationCommand, ApplicationCommandInteraction,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

// mod grim;
mod db_support;
mod images;
mod jeopardy;
mod roll;
mod serenity_impls;
mod spirits_awaken;
mod wumpus;

struct Handler;

const JEOPARDY_CMD: &'static str = "jeopardy";
const RAYZ_CMD: &'static str = "rayz";
const WUMPUS_CMD: &'static str = "htw";
const SPIRITS_CMD: &'static str = "spirits";
const ROLL_CMD: &'static str = "roll";
const DALLE_CMD: &'static str = "dalle";

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
                        .create_option(|option| {
                            option
                                .name("description")
                                .description("A description of the scene in eisenscript.")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
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
                .create_application_command(|commands| {
                    commands
                        .name(ROLL_CMD)
                        .description("Let's roll, baby!")
                        .create_option(|option| {
                            option
                                .name("roll")
                                .description("roll definition")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|commands| {
                    commands
                        .name(DALLE_CMD)
                        .description("Generate images.")
                        .create_option(|option| {
                            option
                                .name("prompt")
                                .description("Some text to feed to the generator.")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
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
                ROLL_CMD => roll(&ctx, command).await,
                DALLE_CMD => generate(&ctx, command).await,
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

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if let Some(vote) = get_vote(add_reaction.emoji.as_data().as_str()) {
            let mut db = get_data::<db_support::DB, _>(&ctx).await;
            let discord_message_id = add_reaction.message_id.0;
            let result = match vote {
                Vote::Up => db.increment_jeopardy_category_post(discord_message_id),
                Vote::Down => db.decrement_jeopardy_category_post(discord_message_id),
            };
            if let Err(err) = result {
                eprintln!("{}", err);
            }
        }
    }

    async fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        if let Some(vote) = get_vote(removed_reaction.emoji.as_data().as_str()) {
            let mut db = get_data::<db_support::DB, _>(&ctx).await;
            let discord_message_id = removed_reaction.message_id.0;
            let result = match vote {
                Vote::Up => db.decrement_jeopardy_category_post(discord_message_id),
                Vote::Down => db.increment_jeopardy_category_post(discord_message_id),
            };
            if let Err(err) = result {
                eprintln!("{}", err);
            }
        }
    }
}

async fn get_data<T, U>(ctx: &Context) -> <T as TypeMapKey>::Value
where
    T: TypeMapKey<Value = U>,
    U: Clone + Send + Sync,
{
    let data_read = ctx.data.read().await;
    data_read
        .get::<T>()
        .expect("Couldn't find data in context.")
        .clone()
}

fn get_vote(reaction: &str) -> Option<jeopardy::Vote> {
    match reaction {
        "👍" => Some(jeopardy::Vote::Up),
        "👎" => Some(jeopardy::Vote::Down),
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    // Login with a bot token from the environment
    let token = std::env::var("BOT_TOKEN").expect("token");
    assert!(serenity::utils::validate_token(&token).is_ok());

    let s3 =
        do_spaces::Client::new(std::env::var("DO_SPACES_SECRET").expect("need DO_SPACES_SECRET"));

    let dalle = if let Ok(url) = std::env::var("DALLE_URL") {
        let r = dalle::Dalle::new(url).await;
        if let Err(err) = &r {
            eprintln!("{}", err);
        }
        r.ok()
    } else {
        None
    };

    let db = tokio::task::block_in_place(|| db::DB::new(db::DB::env_url().unwrap())).unwrap();

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .type_map_insert::<HTWGamesTypeMap>(Default::default())
        .type_map_insert::<db_support::DB>(db)
        .type_map_insert::<serenity_impls::DalleSupport>(dalle)
        .type_map_insert::<serenity_impls::DoSpacesSupport>(s3)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

async fn generate(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    let prompt = command
        .data
        .options
        .first()
        .and_then(|option| option.value.as_ref())
        .and_then(|option| option.as_str())
        .unwrap();
    let s3 = get_data::<serenity_impls::DoSpacesSupport, _>(ctx).await;
    if let Some(dalle) = get_data::<serenity_impls::DalleSupport, _>(ctx).await {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(|data| data.content("Generating images."))
            })
            .await?;

        match dalle.generate(prompt, 4).await {
            Ok(imgs) => {
                let folder = uuid::Uuid::new_v4();
                let uploads = imgs
                    .generated_imgs
                    .into_iter()
                    .enumerate()
                    .map(|(index, img)| {
                        let key = format!("{}/{}.jpeg", folder, index);
                        s3.put_jpeg(key, img)
                    });
                let prompt_upload =
                    s3.put_text(format!("{}/prompt.txt", folder), prompt.to_string());
                let uploads = futures::future::try_join_all(uploads);
                let (prompt_uri, upload_uris) =
                    futures::future::try_join(prompt_upload, uploads).await?;

                // let paths = imgs
                //     .generated_imgs
                //     .iter()
                //     .cloned()
                //     .enumerate()
                //     .map(|(index, data)| {
                //         let filename = format!("{}.{}", index, imgs.generated_imgs_format);
                //         AttachmentType::Bytes {
                //             data: data.into(),
                //             filename,
                //         }
                //     });
                // let mut req = serenity::http::request::RequestBuilder::new(
                //     serenity::http::routing::RouteInfo::EditOriginalInteractionResponse {
                //         application_id: command.application_id.0,
                //         interaction_token: &command.token,
                //     },
                // );
                // req.multipart(Some(serenity::http::multipart::Multipart {
                //     files: paths.into_iter().map(Into::into).collect(),
                //     payload_json: None,
                //     fields: vec![],
                // }))
                // .body(Some(serenity::json::to));
                // ctx.http.fire(req.build()).await?;
                command
                    .edit_original_interaction_response(&ctx.http, |response| {
                        let embeds = upload_uris
                            .into_iter()
                            .map(|url| {
                                let mut embed = CreateEmbed::default();
                                embed.image(url);
                                embed.url(prompt_uri.clone());
                                embed
                            })
                            .collect();
                        response.set_embeds(embeds).content(prompt)
                    })
                    .await?;
            }
            Err(err) => {
                command
                    .edit_original_interaction_response(&ctx.http, |response| response.content(err))
                    .await?;
            }
        }
    } else {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.ephemeral(true).content("Dalle backend not connected.")
                    })
            })
            .await?;
    }

    Ok(())
}

async fn roll(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    let input = command
        .data
        .options
        .iter()
        .find_map(|option| {
            (option.name == "roll").then(|| option.value.as_ref().and_then(|value| value.as_str()))
        })
        .flatten()
        .unwrap_or("");

    let name = if let Some(guild) = command.guild_id {
        command
            .user
            .nick_in(&ctx.http, guild)
            .await
            .unwrap_or(command.user.name.clone())
    } else {
        command.user.name.clone()
    };
    let content = match roll::roll(input) {
        Ok(roll) => format!("{} rolled {}: {}", name, input, roll),
        Err(err) => format!("{}", err),
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

async fn jeopardy(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    let fetch = {
        let mut db = get_data::<db_support::DB, _>(ctx).await;
        move || -> eyre::Result<_> {
            // 100 is a guard against an infinite loop
            (0..100)
                .find_map(|_| {
                    let result = db.random_jeopardy_category();
                    match result {
                        Ok((c, q)) => (q.len() == 5).then(|| Ok((c, q))),
                        Err(err) => Some(Err(err)),
                    }
                })
                .unwrap()
        }
    };
    let result = tokio::task::block_in_place(fetch);

    let (content, category) = match result {
        Ok((category, questions)) => (jeopardy::print(&category, &questions), Some(category)),
        Err(err) => (format!("{}", err), None),
    };

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await?;
    if let Some(category) = category {
        let msg = command.get_interaction_response(&ctx.http).await?;
        tokio::task::block_in_place({
            let mut db = get_data::<db_support::DB, _>(ctx).await;
            move || db.record_jeopardy_category_post(category.id, msg.id.0)
        })?;
    }

    Ok(())
}

async fn rayz(ctx: &Context, command: ApplicationCommandInteraction) -> eyre::Result<()> {
    images::rayz(ctx, command).await
}

#[cfg(test)]
mod tests {}
