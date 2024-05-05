use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::util::env_or_default;

mod util;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum BotCommand {
    #[command(description = "Get help")]
    Help,
    #[command(description = "Call the police")]
    CallPolice,
}

#[tracing::instrument]
async fn handle_help(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
        .reply_to_message_id(msg.id)
        .send()
        .await?;

    Ok(())
}

#[tracing::instrument]
async fn handle_call_police(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, util::call_police_string())
        .reply_to_message_id(msg.id)
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv::dotenv().ok();
    log::info!("Starting call the police bot...");

    let bot = Bot::from_env()
        .set_api_url(reqwest::Url::parse(env_or_default("TELEGRAM_API_URL", "https://api.telegram.org").as_str()).unwrap());

    let handler = Update::filter_message()
        .filter_command::<BotCommand>()
        .branch(dptree::case![BotCommand::Help].endpoint(handle_help))
        .branch(dptree::case![BotCommand::CallPolice].endpoint(handle_call_police));

    Dispatcher::builder(bot, handler)
        .distribution_function(|_| None::<std::convert::Infallible>)
        .build()
        .dispatch()
        .await;
}
