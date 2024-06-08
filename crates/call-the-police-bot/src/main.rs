use std::env;

use anyhow::Context;
use sea_orm::{Database, DatabaseConnection};
use teloxide::{Bot, dptree, update_listeners};
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::{Dispatcher, LoggingErrorHandler, Update};

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
    observability::tracing::init_tracer(
        env!("CARGO_PKG_NAME").into(),
        env!("CARGO_PKG_VERSION").into(),
    );

    log::info!("Starting call the police bot...");

    // create the bot instance
    let bot = Bot::from_env().set_api_url(
        env_or_default("TELEGRAM_API_URL", "https://api.telegram.org")
            .as_str()
            .parse()
            .context("Failed to parse TELEGRAM_API_URL")?,
    );

    // create the update listener
    let update_listener = {
        let webhook_listen_addr = env_or_default("WEBHOOK_LISTEN_ADDR", "0.0.0.0:8080")
            .parse()
            .context("Failed to parse WEBHOOK_LISTEN_ADDR")?;
        log::debug!("webhook_listen_addr: {}", webhook_listen_addr);

        let webhook_url = env_or_default("WEBHOOK_URL", "http://call-the-police-bot:8080")
            .parse()
            .context("Failed to parse WEBHOOK_URL")?;
        log::debug!("webhook_url: {}", webhook_url);

        update_listeners::webhooks::axum(
            bot.clone(),
            update_listeners::webhooks::Options::new(webhook_listen_addr, webhook_url),
        )
        .await
        .context("Failed to create the webhook update listener")?
    };

    // define the handler
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<BotCommand>()
                .branch(dptree::case![BotCommand::Start].endpoint(handle_start))
                .branch(dptree::case![BotCommand::Help].endpoint(handle_help))
                .branch(dptree::case![BotCommand::CallPolice].endpoint(handle_call_police)),
        )
        .branch(Update::filter_inline_query().endpoint(handle_inline_query));

    // init database connection
    let database_connection =
        Database::connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
            .await
            .context("Failed to connect to the database")?;

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![database_connection])
        .distribution_function(|_| None::<std::convert::Infallible>)
        .build()
        .dispatch_with_listener(update_listener, LoggingErrorHandler::new())
        .await;

    Ok(())
}
