# Call The Police Bot

A telegram bot that helps you to call the police.

> It just for fun, don't be serious.

## Usage

- `/help` - Show help message
- `/callpolice` - Call the police, the bot will return a message with the emojis of a police car.

## Run

Before you start, you need to create a bot on Telegram and get the token.

To prepare the environment, you need to install the Cargo and Rust compiler. You can install it
using [rustup](https://rustup.rs/).

After that, you can clone the repository and run the bot.

### Configuration

We provide some environment variables to configure the bot:

- `TELOXIDE_TOKEN` - The token of the bot on Telegram
- `TELEGRAM_API_URL` - The URL of the Telegram API (default: `https://api.telegram.org`)
- `OTEL_EXPORTER_ENDPOINT` - The endpoint of the OpenTelemetry exporter (default: `http://localhost:4317`)
- `OTEL_EXPORTER` - The type of the OpenTelemetry exporter (default: `otlp_grpc`, available: `otlp_grpc`, `otlp_http`)
- `OTEL_SAMPLE_RATE` - The sample rate of the OpenTelemetry exporter (default: `1.0`)
- `RUST_LOG` - The log level of the application (available: `trace`, `debug`, `info`, `warn`, `error`)

## License

This project is licensed under the [GNU Affero General Public License v3.0](LICENSE).
