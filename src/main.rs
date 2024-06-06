use teloxide::{Bot, dptree};
use teloxide::dispatching::{HandlerExt, UpdateFilterExt};
use teloxide::prelude::{Dispatcher, Update};

use crate::handlers::*;
use crate::util::env_or_default;

mod handlers;
mod observability;
mod util;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv::dotenv().ok();
    observability::tracing::init_tracer();
    log::info!("Starting call the police bot...");

    let bot = Bot::from_env().set_api_url(
        reqwest::Url::parse(
            env_or_default("TELEGRAM_API_URL", "https://api.telegram.org").as_str(),
        )
        .unwrap(),
    );

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
        .dispatch()
        .await;
}
