use chrono::offset::Local;
use colored::*;

use models::ver;
use rule::parse_app;
use sea_orm::entity::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, QueryFilter, Select,
};

mod pause;
mod status;
pub use pause::pause;
pub use status::{
    FAILED_KEY, SUCCESS_KEY, SharedStatus, StatusPrinter, StatusRecorder, init_status, print_status,
};

pub const SEPARATOR: &str = "====================================";

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

pub async fn update_app<T: StatusRecorder>(
    app: ver::Model,
    db: &DatabaseConnection,
    status: &T,
) -> anyhow::Result<()> {
    let app_name = app.name.clone();

    match parse_app(&app).await {
        Ok(new_ver) if new_ver != app.version => {
            let mut active_model: ver::ActiveModel = app.into();
            active_model.version = Set(new_ver.clone());
            active_model.updated_at = Set(Some(Local::now()));
            active_model.update(db).await?;

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
