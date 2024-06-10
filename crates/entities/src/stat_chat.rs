use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "stat_chat")]
pub struct Model {
    #[sea_orm(primary_key, not_null)]
    pub chat_id: i64,
    #[sea_orm(primary_key, not_null)]
    pub begin_at: chrono::NaiveDate, // stat for 1 day

    #[sea_orm(default_value = 0, not_null)]
    pub total_emoji_sent: i64, // total police emoji sent
    #[sea_orm(default_value = 0, not_null)]
    pub total_command_triggered: i32, // total command triggered
    #[sea_orm(default_value = 0, not_null)]
    pub total_inline_query_sent: i32, // total inline query sent
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
