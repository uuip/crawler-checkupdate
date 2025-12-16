use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::offset::Local;
use colored::*;

use models::ver;
use rule::parse_app;
use sea_orm::entity::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Select,
};

mod pause;
pub use pause::pause;

pub type SharedStatus = Arc<Mutex<HashMap<&'static str, Vec<String>>>>;

pub const SUCCESS_KEY: &str = "success";
pub const FAILED_KEY: &str = "failed";
pub const SEPARATOR: &str = "====================================";

pub trait StatusPrinter {
    fn get_list(&self, key: &str) -> Vec<String>;
    fn add_to_list(&self, key: &'static str, value: String);
}

impl StatusPrinter for HashMap<&str, Vec<String>> {
    fn get_list(&self, key: &str) -> Vec<String> {
        self.get(key).cloned().unwrap_or_default()
    }

    fn add_to_list(&self, _key: &'static str, _value: String) {
        panic!("Cannot add to non-mutable HashMap")
    }
}

impl StatusPrinter for Arc<Mutex<HashMap<&'static str, Vec<String>>>> {
    fn get_list(&self, key: &str) -> Vec<String> {
        self.lock().unwrap().get(key).cloned().unwrap_or_default()
    }

    fn add_to_list(&self, key: &'static str, value: String) {
        self.lock().unwrap().entry(key).or_default().push(value);
    }
}

#[cfg(feature = "dashmap-support")]
impl StatusPrinter for std::sync::Arc<dashmap::DashMap<&'static str, Vec<String>>> {
    fn get_list(&self, key: &str) -> Vec<String> {
        self.get(key).map(|v| v.clone()).unwrap_or_default()
    }

    fn add_to_list(&self, key: &'static str, value: String) {
        self.entry(key).or_default().push(value);
    }
}

pub fn print_status<T: StatusPrinter>(status: &T) {
    let success = status.get_list(SUCCESS_KEY).join(", ");
    let failed = status.get_list(FAILED_KEY).join(", ");
    println!(
        "成功: {}\n失败: {}",
        if success.is_empty() { "无" } else { &success },
        if failed.is_empty() { "无" } else { &failed }
    );
}

pub fn get_db_path() -> &'static str {
    #[cfg(windows)]
    {
        "sqlite:///C:/Users/sharp/AppData/Local/Programs/checkupdate/ver_tab.db"
    }
    #[cfg(unix)]
    {
        "sqlite:///Users/sharp/ver_tab.db"
    }
}

pub fn init_status() -> SharedStatus {
    Arc::new(Mutex::new(HashMap::from([
        (SUCCESS_KEY, Vec::new()),
        (FAILED_KEY, Vec::new()),
    ])))
}

pub fn query_apps() -> Select<models::VerEntity> {
    #[cfg(windows)]
    {
        models::VerEntity::find().filter(
            ver::Column::Platform
                .eq("Windows")
                .or(ver::Column::Platform.is_null()),
        )
    }
    #[cfg(unix)]
    {
        models::VerEntity::find().filter(
            ver::Column::Platform
                .ne("Windows")
                .or(ver::Column::Platform.is_null()),
        )
    }
}

pub async fn update_app<T: StatusPrinter>(
    app: ver::Model,
    db: DatabaseConnection,
    status: &T,
) -> anyhow::Result<()> {
    let app_name = app.name.clone();

    match parse_app(&app).await {
        Ok(new_ver) if new_ver != app.version => {
            let mut active_model: ver::ActiveModel = app.into();
            active_model.version = Set(new_ver.clone());
            active_model.updated_at = Set(Some(Local::now()));
            active_model.update(&db).await?;

            println!("{} 更新为版本 {}", app_name.green(), new_ver.bright_green());
            status.add_to_list(SUCCESS_KEY, app_name);
            println!("{SEPARATOR}");
            Ok(())
        }
        Ok(new_ver) => {
            println!("{} : {}", app_name.bright_cyan(), new_ver.bright_cyan());
            println!("{SEPARATOR}");
            Ok(())
        }
        Err(e) => {
            eprintln!("{} 获取版本失败:{}", app_name, e);
            status.add_to_list(FAILED_KEY, app_name);
            println!("{SEPARATOR}");
            Err(e)
        }
    }
}
