use anyhow::Context;
use sea_orm::{ActiveValue, DatabaseConnection, prelude::*, QuerySelect};

#[derive(Debug, Clone)]
pub struct Service {
    db: DatabaseConnection,
}

impl Service {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn get_user_current_stat(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Option<entities::stat_user::Model>> {
        entities::stat_user::Entity::find()
            .filter(entities::stat_user::Column::UserId.eq(user_id))
            .filter(entities::stat_user::Column::BeginAt.eq(chrono::Utc::now().naive_utc().date()))
            .one(&self.db)
            .await
            .context("Failed to get record")
    }
}

pub struct UserStatData {
    pub total_emoji_sent: i64,
    pub total_command_triggered: i32,
    pub total_inline_query_sent: i32,
}

pub trait IUserStat {
    /// Get user stat data for the last `days` days.
    ///
    /// # Arguments
    ///
    /// * `user_id`: Telegram user ID
    /// * `days`: Number of days to get the stat data, e.g. 1 for today, 2 for today and yesterday
    ///
    /// returns: `Result<UserStatData, Error>`
    ///
    async fn get_user_stat(&self, user_id: i64, days: i64) -> anyhow::Result<UserStatData>;

    /// Increment the total emoji sent by the user.
    ///
    /// # Arguments
    ///
    /// * `user_id`: Telegram user ID
    /// * `count`: Number of emoji sent
    ///
    /// returns: `Result<(), Error>`
    ///
    async fn increment_total_emoji_sent(&self, user_id: i64, count: i64) -> anyhow::Result<()>;

    /// Increment the total command triggered by the user.
    ///
    /// # Arguments
    ///
    /// * `user_id`: Telegram user ID
    /// * `count`: Number of command triggered
    ///
    /// returns: `Result<(), Error>`
    async fn increment_total_command_triggered(
        &self,
        user_id: i64,
        count: i32,
    ) -> anyhow::Result<()>;

    /// Increment the total inline query sent by the user.
    ///
    /// # Arguments
    ///
    /// * `user_id`: Telegram user ID
    /// * `count`: Number of inline query sent
    ///
    /// returns: `Result<(), Error>`
    async fn increment_total_inline_query_sent(
        &self,
        user_id: i64,
        count: i32,
    ) -> anyhow::Result<()>;
}

impl IUserStat for Service {
    #[tracing::instrument]
    async fn get_user_stat(&self, user_id: i64, days: i64) -> anyhow::Result<UserStatData> {
        if days < 1 {
            return Err(anyhow::anyhow!("Invalid days"));
        }

        let end = chrono::Utc::now().naive_utc().date();
        let begin = end - chrono::Duration::days(days - 1);

        let data = entities::stat_user::Entity::find()
            .filter(entities::stat_user::Column::UserId.eq(user_id))
            .filter(entities::stat_user::Column::BeginAt.between(begin, end))
            .select_only()
            .column_as(
                entities::stat_user::Column::TotalEmojiSent.sum(),
                "total_emoji_sent",
            )
            .column_as(
                entities::stat_user::Column::TotalCommandTriggered.sum(),
                "total_command_triggered",
            )
            .column_as(
                entities::stat_user::Column::TotalInlineQuerySent.sum(),
                "total_inline_query_sent",
            )
            .one(&self.db)
            .await
            .context("Failed to get user stat")?;

        if data.is_none() {
            return Ok(UserStatData {
                total_emoji_sent: 0,
                total_command_triggered: 0,
                total_inline_query_sent: 0,
            });
        }

        let data = data.unwrap();

        Ok(UserStatData {
            total_emoji_sent: data.total_emoji_sent,
            total_command_triggered: data.total_command_triggered,
            total_inline_query_sent: data.total_inline_query_sent,
        })
    }

    #[tracing::instrument]
    async fn increment_total_emoji_sent(&self, user_id: i64, count: i64) -> anyhow::Result<()> {
        let result = self.get_user_current_stat(user_id).await?;

        if let Some(record) = result {
            let mut record: entities::stat_user::ActiveModel = record.into();
            record.total_emoji_sent = ActiveValue::Set(record.total_emoji_sent.unwrap() + count);
            record
                .save(&self.db)
                .await
                .context("Failed to save record")?;
        } else {
            entities::stat_user::ActiveModel {
                user_id: ActiveValue::Set(user_id),
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
        user_id: i64,
        count: i32,
    ) -> anyhow::Result<()> {
        let result = self.get_user_current_stat(user_id).await?;

        if let Some(record) = result {
            let mut record: entities::stat_user::ActiveModel = record.into();
            record.total_command_triggered =
                ActiveValue::Set(record.total_command_triggered.unwrap() + count);
            record
                .save(&self.db)
                .await
                .context("Failed to save record")?;
        } else {
            entities::stat_user::ActiveModel {
                user_id: ActiveValue::Set(user_id),
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
        user_id: i64,
        count: i32,
    ) -> anyhow::Result<()> {
        let result = self.get_user_current_stat(user_id).await?;

        if let Some(record) = result {
            let mut record: entities::stat_user::ActiveModel = record.into();
            record.total_inline_query_sent =
                ActiveValue::Set(record.total_inline_query_sent.unwrap() + count);
            record
                .save(&self.db)
                .await
                .context("Failed to save record")?;
        } else {
            entities::stat_user::ActiveModel {
                user_id: ActiveValue::Set(user_id),
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
