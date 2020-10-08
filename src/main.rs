use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

mod grim;

#[group]
#[commands(ping, new, join, start, draw, end)]
struct General;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = std::env::var("BOT_TOKEN").expect("token");
    assert!(serenity::client::validate_token(&token).is_ok());

    let mut client = Client::new(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;

        data.insert::<GrimGames>(Default::default());
        data.insert::<GrimBuilders>(Default::default());
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

struct GrimGames;
impl TypeMapKey for GrimGames {
    type Value = std::sync::Arc<RwLock<std::collections::HashMap<ChannelId, grim::Grim>>>;
}

struct GrimBuilders;
impl TypeMapKey for GrimBuilders {
    type Value = std::sync::Arc<RwLock<std::collections::HashMap<ChannelId, grim::Builder>>>;
}

#[command]
async fn new(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let data_read = ctx.data.read().await;
        let games_lock = data_read
            .get::<GrimGames>()
            .expect("Expected GrimGames in TypeMap.")
            .clone();
        let games = games_lock.read().await;

        if let Some(game) = games.get(&msg.channel_id) {
            let content = format!("Game by {} in progress.", game.admin.name);
            msg.reply(ctx, content).await?;
            return Ok(());
        }
    }

    {
        let data_write = ctx.data.write().await;
        let games_builders_lock = data_write
            .get::<GrimBuilders>()
            .expect("Expected GrimBuilders in TypeMap.")
            .clone();
        let mut builders = games_builders_lock.write().await;

        if let Some(builder) = builders.get(&msg.channel_id) {
            let content = format!("Game by {} waiting for players.", builder.creator.name);
            msg.reply(ctx, content).await?;
        } else {
            builders.insert(msg.channel_id, grim::Builder::new(msg.author.clone()));
            let content = format!("New game started! Waiting for players.");
            msg.reply(ctx, content).await?;
        }
    }

    Ok(())
}

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let data_write = ctx.data.write().await;
        let games_builders_lock = data_write
            .get::<GrimBuilders>()
            .expect("Expected GrimBuilders in TypeMap.")
            .clone();
        let mut builders = games_builders_lock.write().await;

        if let Some(builder) = builders.get_mut(&msg.channel_id) {
            builder.add_player(&msg.author);
            let players = builder
                .players()
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Box<_>>()
                .join(", ");
            let content = format!("Added! Current players: {}.", players);
            msg.reply(ctx, content).await?;
        } else {
            let content = format!("No game waiting for players!");
            msg.reply(ctx, content).await?;
        }
    }

    Ok(())
}

#[command]
async fn start(ctx: &Context, msg: &Message) -> CommandResult {
    let game = {
        let data_write = ctx.data.write().await;
        let games_builders_lock = data_write
            .get::<GrimBuilders>()
            .expect("Expected GrimBuilders in TypeMap.")
            .clone();
        let mut builders = games_builders_lock.write().await;

        if let Some(builder) = builders.get(&msg.channel_id) {
            if msg.author == builder.creator {
                let builder = builders.remove(&msg.channel_id).unwrap();
                Some(builder.ready())
            } else {
                None
            }
        } else {
            let content = format!("No game to start!");
            msg.reply(ctx, content).await?;
            None
        }
    };

    if let Some(game) = game {
        let data_write = ctx.data.write().await;
        let games_lock = data_write
            .get::<GrimGames>()
            .expect("Expected GrimGames in TypeMap.")
            .clone();
        let mut games = games_lock.write().await;

        games.insert(msg.channel_id, game);
        let content = format!("Game started!");
        msg.reply(ctx, content).await?;
    }

    Ok(())
}

#[command]
async fn draw(ctx: &Context, msg: &Message) -> CommandResult {
    let data_write = ctx.data.write().await;
    let games_lock = data_write
        .get::<GrimGames>()
        .expect("Expected GrimGames in TypeMap.")
        .clone();
    let mut games = games_lock.write().await;

    if let Some(game) = games.get_mut(&msg.channel_id) {
        if game.is_player(&msg.author) {
            if let Some(card) = game.draw() {
                let content = format!(
                    "Drew {}! {} cards remaining!",
                    card.description(),
                    game.deck.len()
                );
                msg.reply(ctx, content).await?;
            } else {
                let content = format!("Deck is out of cards!");
                msg.reply(ctx, content).await?;
            }
        }
    }

    Ok(())
}

#[command]
async fn end(ctx: &Context, msg: &Message) -> CommandResult {
    {
        let data_write = ctx.data.write().await;
        let games_lock = data_write
            .get::<GrimGames>()
            .expect("Expected GrimGames in TypeMap.")
            .clone();
        let mut games = games_lock.write().await;

        if let Some(game) = games.get(&msg.channel_id) {
            if game.admin == msg.author {
                let game = games.remove(&msg.channel_id).unwrap();
                let content = format!("Game ended by {}.", game.admin.name);
                msg.reply(ctx, content).await?;
            }
        }
    }

    {
        let data_write = ctx.data.write().await;
        let games_builders_lock = data_write
            .get::<GrimBuilders>()
            .expect("Expected GrimBuilders in TypeMap.")
            .clone();
        let mut builders = games_builders_lock.write().await;

        if let Some(builder) = builders.get(&msg.channel_id) {
            if builder.creator == msg.author {
                let builder = builders.remove(&msg.channel_id).unwrap();
                let content = format!("Game ended by {}.", builder.creator.name);
                msg.reply(ctx, content).await?;
            }
        }
    }

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    tokio::time::delay_for(tokio::time::Duration::from_secs(2)).await;

    msg.reply(ctx, "Pong2!").await?;

    Ok(())
}
