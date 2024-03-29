use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "ver_tab")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub name: String,
    pub ver: String,
    pub url: String,
    pub newversion: Option<String>,
    pub json: i8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
