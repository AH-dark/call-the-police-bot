use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::{Dispatcher, LoggingErrorHandler, Update};
use teloxide::{dptree, update_listeners, Bot};

use crate::handlers::*;
use crate::util::env_or_default;

mod handlers;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match dotenv::dotenv() {
        Ok(_) => log::info!("Loaded .env file"),
        Err(e) => log::warn!("Failed to load .env file: {}", e),
    }
    pretty_env_logger::init();
    observability::init(
        env!("CARGO_PKG_NAME").into(),
        env!("CARGO_PKG_VERSION").into(),
    )
    .expect("Failed to initialize observability");

    log::info!("Starting call the police bot...");

    let bot = Bot::from_env().set_api_url(
        reqwest::Url::parse(
            env_or_default("TELEGRAM_API_URL", "https://api.telegram.org").as_str(),
        )
        .unwrap(),
    );

    let update_listener = {
        let webhook_listen_addr = env_or_default("WEBHOOK_LISTEN_ADDR", "0.0.0.0:8080")
            .parse()
            .unwrap();
        log::debug!("webhook_listen_addr: {}", webhook_listen_addr);

        let webhook_url = env_or_default("WEBHOOK_URL", "http://call-the-police-bot:8080")
            .parse()
            .unwrap();
        log::debug!("webhook_url: {}", webhook_url);

        update_listeners::webhooks::axum(
            bot.clone(),
            update_listeners::webhooks::Options::new(webhook_listen_addr, webhook_url),
        )
        .await?
    };

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<BotCommand>()
                .branch(dptree::case![BotCommand::Start].endpoint(handle_start))
                .branch(dptree::case![BotCommand::Help].endpoint(handle_help))
                .branch(dptree::case![BotCommand::CallPolice].endpoint(handle_call_police)),
        )
        .branch(Update::filter_inline_query().endpoint(handle_inline_query));

    Dispatcher::builder(bot, handler)
        .distribution_function(|_| None::<std::convert::Infallible>)
        .build()
        .dispatch_with_listener(update_listener, LoggingErrorHandler::new())
        .await;

    Ok(())
}
