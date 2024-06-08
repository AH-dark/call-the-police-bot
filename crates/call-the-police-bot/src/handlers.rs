use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{
    InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
};
use teloxide::utils::command::BotCommands;

use crate::util;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum BotCommand {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Get help")]
    Help,
    #[command(description = "Call the police")]
    CallPolice,
}

#[tracing::instrument]
pub async fn handle_start(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(
        msg.chat.id,
        "Hello! I'm the call the police bot. You can view available commands by typing /help.",
    )
    .send()
    .await?;

    Ok(())
}

#[tracing::instrument]
pub async fn handle_help(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
        .reply_to_message_id(msg.id)
        .send()
        .await?;

    Ok(())
}

#[tracing::instrument]
pub async fn handle_call_police(bot: Bot, msg: Message) -> ResponseResult<()> {
    let times = util::rand_num(8, 96);
    bot.send_message(msg.chat.id, util::call_police_string(times))
        .reply_to_message_id(msg.id)
        .send()
        .await?;

    Ok(())
}

#[tracing::instrument]
pub async fn handle_inline_query(bot: Bot, query: InlineQuery) -> ResponseResult<()> {
    let times = query
        .query
        .parse::<usize>()
        .unwrap_or_else(|_| util::rand_num(8, 96))
        .max(1)
        .min(4096);

    let results = vec![InlineQueryResult::Article(
        InlineQueryResultArticle::new(
            "call-the-police",
            "Call the police",
            InputMessageContent::Text(InputMessageContentText::new(util::call_police_string(
                times,
            ))),
        )
        .description("Generate a random string of police emojis.")
        .thumb_url(
            "https://raw.githubusercontent.com/AH-dark/call-the-police-bot/main/assets/call_back_query_thumb.jpeg"
                .parse()
                .expect("valid URL"),
        ),
    )];

    bot.answer_inline_query(query.id, results)
        .is_personal(true)
        .cache_time(0)
        .send()
        .await?;

    Ok(())
}
