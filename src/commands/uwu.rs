use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use uwuifier::uwuify_str_sse;

#[command]
#[aliases("uwu")]
#[description = "Uwuifies the given text."]
pub async fn uwuify(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = if args.is_empty() {
        match &msg.referenced_message {
            Some(m) => m.content_safe(ctx),
            None => {
                msg.reply(ctx, ":x: No message given.").await?;
                return Ok(());
            }
        }
    } else {
        args.rest().to_string()
    };

    msg.channel_id
        .send_message(ctx, |m| m.content(uwuify_str_sse(&text)))
        .await?;

    Ok(())
}
