use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "stat_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: u64,
    #[sea_orm(primary_key)]
    pub begin_at: chrono::NaiveDate, // stat for 1 day

    #[sea_orm(default_value = 0)]
    pub total_emoji_sent: u64, // total police emoji sent
    #[sea_orm(default_value = 0)]
    pub total_command_triggered: u32, // total command triggered
    #[sea_orm(default_value = 0)]
    pub total_inline_query_sent: u32, // total inline query sent
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
