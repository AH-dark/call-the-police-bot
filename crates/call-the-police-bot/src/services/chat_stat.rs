use anyhow::Context;
use sea_orm::{ActiveValue, DatabaseConnection, FromQueryResult, prelude::*, QuerySelect};
use sea_orm::sea_query::Alias;

#[derive(Debug, Clone)]
pub struct Service {
    db: DatabaseConnection,
}

impl Service {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn get_chat_current_stat(
        &self,
        chat_id: i64,
    ) -> anyhow::Result<Option<entities::stat_chat::Model>> {
        entities::stat_chat::Entity::find()
            .filter(entities::stat_chat::Column::ChatId.eq(chat_id))
            .filter(entities::stat_chat::Column::BeginAt.eq(chrono::Utc::now().naive_utc().date()))
            .one(&self.db)
            .await
            .context("Failed to get record")
    }
}

#[derive(FromQueryResult, Debug, Clone, Copy)]
pub struct ChatStatResult {
    pub total_emoji_sent: Option<i64>,
    pub total_command_triggered: Option<i32>,
    pub total_inline_query_sent: Option<i32>,
}

#[derive(Debug, Clone, Copy)]
pub struct ChatStatData {
    pub total_emoji_sent: i64,
    pub total_command_triggered: i32,
    pub total_inline_query_sent: i32,
}

pub trait IChatStat {
    /// Get chat stat data for the last `days` days.
    ///
    /// # Arguments
    ///
    /// * `chat_id`: Telegram chat ID
    /// * `days`: Number of days to get the stat data, e.g. 1 for today, 2 for today and yesterday
    ///
    /// returns: `Result<ChatStatData, Error>`
    ///
    async fn get_chat_stat(&self, chat_id: i64, days: i64) -> anyhow::Result<ChatStatData>;

    /// Increment the total emoji sent by the chat.
    ///
    /// # Arguments
    ///
    /// * `chat_id`: Telegram chat ID
    /// * `count`: Number of emoji sent
    ///
    /// returns: `Result<(), Error>`
    ///
    async fn increment_total_emoji_sent(&self, chat_id: i64, count: i64) -> anyhow::Result<()>;

    /// Increment the total command triggered by the chat.
    ///
    /// # Arguments
    ///
    /// * `chat_id`: Telegram chat ID
    /// * `count`: Number of command triggered
    ///
    /// returns: `Result<(), Error>`
    async fn increment_total_command_triggered(
        &self,
        chat_id: i64,
        count: i32,
    ) -> anyhow::Result<()>;

    /// Increment the total inline query sent by the chat.
    ///
    /// # Arguments
    ///
    /// * `chat_id`: Telegram chat ID
    /// * `count`: Number of inline query sent
    ///
    /// returns: `Result<(), Error>`
    async fn increment_total_inline_query_sent(
        &self,
        chat_id: i64,
        count: i32,
    ) -> anyhow::Result<()>;
}

impl IChatStat for Service {
    #[tracing::instrument]
    async fn get_chat_stat(&self, chat_id: i64, days: i64) -> anyhow::Result<ChatStatData> {
        if days < 1 {
            return Err(anyhow::anyhow!("Invalid days"));
        }

        let end = chrono::Utc::now().naive_utc().date();
        let begin = end - chrono::Duration::days(days - 1);

        let data = entities::stat_chat::Entity::find()
            .filter(entities::stat_chat::Column::ChatId.eq(chat_id))
            .filter(entities::stat_chat::Column::BeginAt.between(begin, end))
            .select_only()
            .column_as(
                entities::stat_chat::Column::TotalEmojiSent
                    .sum()
                    .cast_as(Alias::new("bigint")),
                "total_emoji_sent",
            )
            .column_as(
                entities::stat_chat::Column::TotalCommandTriggered
                    .sum()
                    .cast_as(Alias::new("integer")),
                "total_command_triggered",
            )
            .column_as(
                entities::stat_chat::Column::TotalInlineQuerySent
                    .sum()
                    .cast_as(Alias::new("integer")),
                "total_inline_query_sent",
            )
            .into_model::<ChatStatResult>()
            .one(&self.db)
            .await
            .context("Failed to get chat stat")?;

        if data.is_none() {
            return Ok(ChatStatData {
                total_emoji_sent: 0,
                total_command_triggered: 0,
                total_inline_query_sent: 0,
            });
        }

        Ok(ChatStatData {
            total_emoji_sent: data.unwrap().total_emoji_sent.unwrap_or(0),
            total_command_triggered: data.unwrap().total_command_triggered.unwrap_or(0),
            total_inline_query_sent: data.unwrap().total_inline_query_sent.unwrap_or(0),
        })
    }

    #[tracing::instrument]
    async fn increment_total_emoji_sent(&self, chat_id: i64, count: i64) -> anyhow::Result<()> {
        let result = self.get_chat_current_stat(chat_id).await?;

        if let Some(record) = result {
            let mut record: entities::stat_chat::ActiveModel = record.into();
            record.total_emoji_sent = ActiveValue::Set(record.total_emoji_sent.unwrap() + count);
            record
                .save(&self.db)
                .await
                .context("Failed to save record")?;
        } else {
            entities::stat_chat::ActiveModel {
                chat_id: ActiveValue::Set(chat_id),
                begin_at: ActiveValue::Set(chrono::Utc::now().naive_utc().date()),
                total_emoji_sent: ActiveValue::Set(count),
                total_command_triggered: ActiveValue::Set(0),
                total_inline_query_sent: ActiveValue::Set(0),
            }
            .insert(&self.db)
            .await
            .context("Failed to create record")?;
        }

        Ok(())
    }

    #[tracing::instrument]
    async fn increment_total_command_triggered(
        &self,
        chat_id: i64,
        count: i32,
    ) -> anyhow::Result<()> {
        let result = self.get_chat_current_stat(chat_id).await?;

        if let Some(record) = result {
            let mut record: entities::stat_chat::ActiveModel = record.into();
            record.total_command_triggered =
                ActiveValue::Set(record.total_command_triggered.unwrap() + count);
            record
                .save(&self.db)
                .await
                .context("Failed to save record")?;
        } else {
            entities::stat_chat::ActiveModel {
                chat_id: ActiveValue::Set(chat_id),
                begin_at: ActiveValue::Set(chrono::Utc::now().naive_utc().date()),
                total_emoji_sent: ActiveValue::Set(0),
                total_command_triggered: ActiveValue::Set(count),
                total_inline_query_sent: ActiveValue::Set(0),
            }
            .insert(&self.db)
            .await
            .context("Failed to create record")?;
        }

        Ok(())
    }

    #[tracing::instrument]
    async fn increment_total_inline_query_sent(
        &self,
        chat_id: i64,
        count: i32,
    ) -> anyhow::Result<()> {
        let result = self.get_chat_current_stat(chat_id).await?;

        if let Some(record) = result {
            let mut record: entities::stat_chat::ActiveModel = record.into();
            record.total_inline_query_sent =
                ActiveValue::Set(record.total_inline_query_sent.unwrap() + count);
            record
                .save(&self.db)
                .await
                .context("Failed to save record")?;
        } else {
            entities::stat_chat::ActiveModel {
                chat_id: ActiveValue::Set(chat_id),
                begin_at: ActiveValue::Set(chrono::Utc::now().naive_utc().date()),
                total_emoji_sent: ActiveValue::Set(0),
                total_command_triggered: ActiveValue::Set(0),
                total_inline_query_sent: ActiveValue::Set(count),
            }
            .insert(&self.db)
            .await
            .context("Failed to create record")?;
        }

        Ok(())
    }
}
