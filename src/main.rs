mod commands;

use std::env;
use std::{collections::HashSet, sync::Arc};

use dotenv::dotenv;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};

use commands::*;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[group]
#[commands(ping)]
struct General;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(
            "connected as {}#{} ({})",
            ready.user.name, ready.user.discriminator, ready.user.id
        );
    }
}

#[tokio::main]
async fn main() {
    dotenv().expect("failed to load .env");

    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info,serenity::client::bridge::gateway=warn");
    }

    tracing_subscriber::fmt::init();

    let token = env::var("PLUTO_BOT_TOKEN").expect("no token found in environment");

    let http = Http::new(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("~").on_mention(Some(bot_id)))
        .help(&HELP_COMMAND)
        .group(&GENERAL_GROUP);
    let intents = GatewayIntents::all();

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("could not register ctrl+c handler");

        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(err) = client.start().await {
        error!("client error: {}", err);
    }
}
