use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::Arc;
use structopt::StructOpt;

mod grim;

#[group]
#[commands(grim)]
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

type GrimHashMap<T> = std::collections::HashMap<String, T>;
type GrimHashMapIndirect<T> = GrimHashMap<Arc<RwLock<T>>>;

struct GrimGames;
impl TypeMapKey for GrimGames {
    type Value = Arc<RwLock<GrimHashMapIndirect<grim::Game>>>;
}

struct GrimBuilders;
impl TypeMapKey for GrimBuilders {
    type Value = Arc<RwLock<GrimHashMap<grim::Builder>>>;
}

async fn get_data<T, U>(ctx: &Context) -> <T as TypeMapKey>::Value
where
    T: TypeMapKey<Value = Arc<RwLock<U>>>,
    U: Send + Sync,
{
    let data_read = ctx.data.read().await;
    Arc::clone(
        data_read
            .get::<T>()
            .expect("Couldn't find data in context."),
    )
}

async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let key = msg.channel(ctx).await.unwrap().to_string();

    {
        let games_builders_lock = get_data::<GrimBuilders, _>(ctx).await;
        let mut builders = games_builders_lock.write().await;

        if let Some(builder) = builders.get_mut(&key) {
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
            let content = format!("No game waiting for players.");
            msg.reply(ctx, content).await?;
        }
    }

    Ok(())
}

async fn draw(ctx: &Context, msg: &Message) -> CommandResult {
    let key = msg.channel(ctx).await.unwrap().to_string();

    let games_lock = get_data::<GrimGames, _>(ctx).await;
    let games = games_lock.write().await;

    if let Some(mut game) = get(&games, &key).await {
        if game.player_position(&msg.author).is_some() {
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

async fn die(ctx: &Context, msg: &Message) -> CommandResult {
    let key = msg.channel(ctx).await.unwrap().to_string();

    let games_lock = get_data::<GrimGames, _>(ctx).await;
    let games = games_lock.write().await;

    if let Some(mut game) = get(&games, &key).await {
        if let Some(index) = game.player_position(&msg.author) {
            let player = game.players.remove(index);
            game.reset();
            let content = format!("{} removed from game.", player.user.name);
            msg.reply(ctx, content).await?;
        }
    }

    Ok(())
}

async fn cards(ctx: &Context, msg: &Message) -> CommandResult {
    let key = msg.channel(ctx).await.unwrap().to_string();

    let games_lock = get_data::<GrimGames, _>(ctx).await;
    let games = games_lock.write().await;

    if let Some(game) = get(&games, &key).await {
        let content = format!("{} cards remaing in deck.", game.deck.len());
        msg.reply(ctx, content).await?;
    }

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Grim", about = "Grim game helper.")]
enum GrimCmd {
    #[structopt(about = "Start a new game.")]
    New { custom_id: Option<String> },
    #[structopt(about = "Join a waiting game.")]
    Join,
    #[structopt(about = "Start a waiting game.")]
    Start,
    #[structopt(about = "Kill your user.")]
    Die,
    #[structopt(about = "Draw a card.")]
    Draw,
    #[structopt(about = "See card status.")]
    Cards,
    #[structopt(about = "Reshuffle deck for active game.")]
    Shuffle {
        custom_id: Option<String>,
        ratio: Option<f32>,
    },
    #[structopt(about = "End an in-progress game.")]
    End,
}

impl GrimCmd {
    pub async fn execute(self, ctx: GrimContext<'_>) -> CommandResult {
        match self {
            GrimCmd::New { custom_id } => ctx.handle_new(custom_id).await,
            GrimCmd::Join => ctx.handle_join().await,
            GrimCmd::End => ctx.handle_end().await,
            GrimCmd::Die => ctx.handle_die().await,
            GrimCmd::Cards => ctx.handle_cards().await,
            GrimCmd::Draw => ctx.handle_draw().await,
            GrimCmd::Start => ctx.handle_start().await,
            GrimCmd::Shuffle { custom_id, ratio } => ctx.handle_shuffle(custom_id, ratio).await,
        }
    }
}

async fn get<'a, T>(
    map: &'a tokio::sync::RwLockWriteGuard<'a, GrimHashMapIndirect<T>>,
    key: &String,
) -> Option<tokio::sync::RwLockWriteGuard<'a, T>> {
    if let Some(v) = map.get(key) {
        Some(v.write().await)
    } else {
        None
    }
}

struct GrimGlobalState {
    games: Arc<RwLock<GrimHashMapIndirect<grim::Game>>>,
    builders: Arc<RwLock<GrimHashMap<grim::Builder>>>,
}

type GrimGameLookup<'a> = tokio::sync::RwLockWriteGuard<'a, GrimHashMapIndirect<grim::Game>>;
type GrimGame<'a> = tokio::sync::RwLockWriteGuard<'a, grim::Game>;

impl GrimGlobalState {
    pub async fn games_mut(&self) -> GrimGameLookup<'_> {
        self.games.write().await
    }

    pub async fn builders(&self) -> tokio::sync::RwLockReadGuard<'_, GrimHashMap<grim::Builder>> {
        self.builders.read().await
    }

    pub async fn builders_mut(
        &self,
    ) -> tokio::sync::RwLockWriteGuard<'_, GrimHashMap<grim::Builder>> {
        self.builders.write().await
    }
}

struct GrimContext<'a> {
    pub ctx: &'a Context,
    pub msg: &'a Message,
    pub state: GrimGlobalState,
}

impl<'a> GrimContext<'a> {
    pub async fn new(ctx: &'a Context, msg: &'a Message) -> GrimContext<'a> {
        let games = get_data::<GrimGames, _>(ctx).await;
        let builders = get_data::<GrimBuilders, _>(ctx).await;

        Self {
            ctx,
            msg,
            state: GrimGlobalState { games, builders },
        }
    }

    pub async fn channel_name(&self) -> String {
        self.msg
            .channel(self.ctx)
            .await
            .map(|c| c.to_string())
            .unwrap()
    }

    pub async fn handle_new(&self, custom_id: Option<String>) -> CommandResult {
        let channel_name = self.channel_name().await;

        {
            let games = self.state.games_mut().await;

            let game_lock = if let Some(custom_id) = &custom_id {
                if let Some(games_lock) = games.get(custom_id) {
                    Some(games_lock)
                } else {
                    games.get(&channel_name)
                }
            } else {
                None
            };

            if let Some(game_lock) = game_lock {
                let game = game_lock.read().await;
                let content = format!("Game by {} in progress.", game.admin.name);
                self.msg.reply(self.ctx, content).await?;
                return Ok(());
            }
        }

        {
            let mut builders = self.state.builders_mut().await;

            if let Some(builder) = builders.get(&channel_name) {
                let content = format!("Game by {} waiting for players.", builder.creator.name);
                self.msg.reply(self.ctx, content).await?;
            } else {
                let creator = self.msg.author.clone();
                let builder = match custom_id {
                    None => grim::Builder::new(creator),
                    Some(custom_id) => grim::Builder::new_with_custom_id(creator, custom_id),
                };
                builders.insert(channel_name, builder);
                let content = format!("New game started! Waiting for players.");
                self.msg.reply(self.ctx, content).await?;
            }
        }

        Ok(())
    }

    pub async fn handle_join(&self) -> CommandResult {
        join(self.ctx, self.msg).await
    }

    pub async fn handle_end(&self) -> CommandResult {
        let channel_name = self.channel_name().await;

        {
            let mut games = self.state.games_mut().await;

            let admin = get(&games, &channel_name)
                .await
                .map(|game| game.admin.clone());
            if let Some(admin) = admin {
                if admin == self.msg.author {
                    let game_lock = games.remove(&channel_name).unwrap();
                    let game = game_lock.read().await;
                    if let Some(custom_id) = &game.custom_id {
                        games.remove(custom_id);
                    }
                    let content = format!("Game ended by {}.", game.admin.name);
                    self.msg.reply(self.ctx, content).await?;
                }
            }
        }

        {
            let mut builders = self.state.builders_mut().await;

            if let Some(builder) = builders.get(&channel_name) {
                if builder.creator == self.msg.author {
                    let builder = builders.remove(&channel_name).unwrap();
                    let content = format!("Game ended by {}.", builder.creator.name);
                    self.msg.reply(self.ctx, content).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn handle_die(&self) -> CommandResult {
        die(self.ctx, self.msg).await
    }

    pub async fn handle_cards(&self) -> CommandResult {
        cards(self.ctx, self.msg).await
    }

    pub async fn handle_draw(&self) -> CommandResult {
        draw(self.ctx, self.msg).await
    }

    pub async fn handle_start(&self) -> CommandResult {
        let channel_name = self.channel_name().await;

        let game = {
            let mut builders = self.state.builders_mut().await;

            if let Some(builder) = builders.get(&channel_name) {
                if self.msg.author == builder.creator {
                    let builder = builders.remove(&channel_name).unwrap();
                    Some(builder.ready())
                } else {
                    None
                }
            } else {
                let content = format!("No game to start!");
                self.msg.reply(self.ctx, content).await?;
                None
            }
        };

        if let Some(game) = game {
            let mut games = self.state.games_mut().await;

            let custom_id = game.custom_id.clone();
            let game = Arc::new(RwLock::new(game));
            if let Some(custom_id) = custom_id {
                games.insert(custom_id, Arc::clone(&game));
            }
            games.insert(channel_name, game);
            let content = format!("Game started!");
            self.msg.reply(self.ctx, content).await?;
        }

        Ok(())
    }

    pub async fn handle_shuffle(
        &self,
        custom_id: Option<String>,
        ratio: Option<f32>,
    ) -> CommandResult {
        let key = custom_id.unwrap_or(self.channel_name().await);
        let games = self.state.games_mut().await;

        if let Some(mut game) = get(&games, &key).await {
            game.reset();
            let content = format!("Deck shuffled succesfully.");
            self.msg.reply(self.ctx, content).await?;
        }

        Ok(())
    }
}

#[command]
async fn grim(ctx: &Context, msg: &Message) -> CommandResult {
    let args = msg
        .content
        .split_ascii_whitespace()
        .map(Into::<std::ffi::OsString>::into);
    match GrimCmd::from_iter_safe(args) {
        Ok(cmd) => {
            let ctx = GrimContext::new(ctx, msg).await;
            cmd.execute(ctx).await?;
        }
        Err(e) => {
            msg.reply(ctx, e.message).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
