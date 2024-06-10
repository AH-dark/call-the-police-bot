use anyhow::Context;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{
    InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
};
use teloxide::utils::command::BotCommands;

use crate::{services, util};
use crate::services::chat_stat::IChatStat;
use crate::services::user_stat::IUserStat;
use crate::util::rand_num;

#[derive(BotCommands, Clone, Debug, PartialEq, Eq)]
#[command(
    rename_rule = "snake_case",
    description = "These commands are supported:"
)]
pub enum BotCommand {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Get help")]
    Help,
    #[command(description = "Call the police", rename = "callpolice")]
    CallPolice,
    #[command(description = "Get user stat")]
    Stat,
    #[command(description = "Get chat stat")]
    ChatStat,
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
    chat_stat_service: services::chat_stat::Service,
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

    chat_stat_service
        .increment_total_command_triggered(msg.chat.id.0, 1)
        .await
        .map_err(|e| {
            log::warn!("Failed to increment total command triggered: {:?}", e);
        })
        .ok();

    chat_stat_service
        .increment_total_emoji_sent(msg.chat.id.0, times as i64)
        .await
        .map_err(|e| {
            log::warn!("Failed to increment total emoji sent: {:?}", e);
        })
        .ok();

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
pub async fn handle_stat(
    bot: Bot,
    msg: Message,
    user_stat_service: services::user_stat::Service,
) -> anyhow::Result<()> {
    let user_id = msg
        .from()
        .map(|from| from.id.0 as i64)
        .ok_or_else(|| anyhow::anyhow!("No user info"))?;

    let time_range = msg.text().map_or(7, |text| {
        text.split_whitespace()
            .nth(1)
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(7)
    });

    let data = user_stat_service
        .get_user_stat(user_id, time_range)
        .await
        .context("Failed to get user stat")?;

    bot.send_message(
        msg.chat.id,
        format!(
            r#"
            Last {} days:
            - Total emoji sent: {}
            - Total command triggered: {}
            - Total inline query sent: {}
            "#,
            time_range,
            data.total_emoji_sent,
            data.total_command_triggered,
            data.total_inline_query_sent,
        )
        .trim()
        .split('\n')
        .map(|s| s.trim())
        .collect::<Vec<&str>>()
        .join("\n"),
    )
    .reply_to_message_id(msg.id)
    .send()
    .await
    .context("Failed to send message")?;

    Ok(())
}

#[tracing::instrument]
pub async fn handle_chat_stat(
    bot: Bot,
    msg: Message,
    chat_stat_service: services::chat_stat::Service,
) -> anyhow::Result<()> {
    let chat_id = msg.chat.id.0;

    let time_range = msg.text().map_or(7, |text| {
        text.split_whitespace()
            .nth(1)
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(7)
    });

    let data = chat_stat_service
        .get_chat_stat(chat_id, time_range)
        .await
        .context("Failed to get chat stat")?;

    bot.send_message(
        msg.chat.id,
        format!(
            r#"
            Last {} days:
            - Total emoji sent: {}
            - Total command triggered: {}
            - Total inline query sent: {}
            "#,
            time_range,
            data.total_emoji_sent,
            data.total_command_triggered,
            data.total_inline_query_sent,
        )
        .trim()
        .split('\n')
        .map(|s| s.trim())
        .collect::<Vec<&str>>()
        .join("\n"),
    )
    .reply_to_message_id(msg.id)
    .send()
    .await
    .context("Failed to send message")?;

    Ok(())
}
