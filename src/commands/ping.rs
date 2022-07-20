use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::Color;

use crate::ShardManagerContainer;

#[command]
#[aliases("latency")]
#[description = "Retrieves the current shard's latency."]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(sm) => sm,
        None => {
            msg.reply(ctx, ":x: Failed to retrieve shard manager.")
                .await?;
            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, ":x: Failed to retrieve shard.").await?;
            return Ok(());
        }
    };

    let latency = runner
        .latency
        .map(|d| format!("{:?}", d))
        .unwrap_or("N/A".to_string());

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Pong!")
                    .description(&format!("Latency: {}", latency))
                    .timestamp(Timestamp::now())
                    .color(Color::FOOYOO)
            })
        })
        .await?;

    Ok(())
}
