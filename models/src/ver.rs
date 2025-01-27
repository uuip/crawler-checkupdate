use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "ver_tab")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub name: String,
    pub version: String,
    pub url: String,
    pub version_rule: Option<String>,
    pub check_type: String,
    pub updated_at: Option<ChronoDateTimeLocal>,
    pub platform: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
