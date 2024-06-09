use anyhow::Context;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{
    InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
};
use teloxide::utils::command::BotCommands;

use crate::{services, util};
use crate::services::user_stat::IUserStat;
use crate::util::rand_num;

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
pub async fn handle_start(bot: Bot, msg: Message) -> anyhow::Result<()> {
    bot.send_message(
        msg.chat.id,
        "Hello! I'm the call the police bot. You can view available commands by typing /help.",
    )
    .send()
    .await
    .context("Failed to send message")?;

    Ok(())
}

#[tracing::instrument]
pub async fn handle_help(bot: Bot, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
        .reply_to_message_id(msg.id)
        .send()
        .await
        .context("Failed to send message")?;

    Ok(())
}

#[tracing::instrument]
pub async fn handle_call_police(
    bot: Bot,
    msg: Message,
    user_stat_service: services::user_stat::Service,
) -> anyhow::Result<()> {
    let times = util::rand_num(8, 96);
    bot.send_message(msg.chat.id, util::call_police_string(times))
        .reply_to_message_id(msg.id)
        .send()
        .await
        .context("Failed to send message")?;

    if let Some(from) = msg.from() {
        user_stat_service
            .increment_total_command_triggered(from.id.0 as i64, 1)
            .await
            .map_err(|e| {
                log::warn!("Failed to increment total command triggered: {:?}", e);
            })
            .ok();

        user_stat_service
            .increment_total_emoji_sent(from.id.0 as i64, times as i64)
            .await
            .map_err(|e| {
                log::warn!("Failed to increment total emoji sent: {:?}", e);
            })
            .ok();
    }

    Ok(())
}

#[tracing::instrument]
pub async fn handle_inline_query(bot: Bot, query: InlineQuery) -> anyhow::Result<()> {
    let times = match query.query.parse::<u64>() {
        Ok(times) if times > 0 && times <= 4096 => times,
        Ok(times) if times > 4096 => 4096,
        Err(e) => match e.kind() {
            std::num::IntErrorKind::Empty => rand_num(8, 96),
            std::num::IntErrorKind::NegOverflow => 8,
            std::num::IntErrorKind::PosOverflow => 4096,
            _ => {
                bot.answer_inline_query(query.id, vec![]).send().await?;
                return Err(anyhow::anyhow!("Invalid query"));
            }
        },
        _ => {
            bot.answer_inline_query(query.id, vec![]).send().await?;
            return Err(anyhow::anyhow!("Invalid query"));
        }
    };

    let results = vec![InlineQueryResult::Article(
        InlineQueryResultArticle::new(
            "call-the-police",
            "Call the police",
            InputMessageContent::Text(InputMessageContentText::new(util::call_police_string(times))),
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
        .await
        .context("Failed to answer inline query")?;

    Ok(())
}

#[tracing::instrument]
pub async fn chosen_inline_result_handler(
    chosen_inline_result: ChosenInlineResult,
    user_stat_service: services::user_stat::Service,
) -> anyhow::Result<()> {
    user_stat_service
        .increment_total_inline_query_sent(chosen_inline_result.from.id.0 as i64, 1)
        .await
        .map_err(|e| {
            log::warn!("Failed to increment total inline query sent: {:?}", e);
        })
        .ok();

    Ok(())
}
