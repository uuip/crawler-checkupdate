use colored::*;
use common::pause;
use dashmap::DashMap;
use futures::StreamExt;
use models::{VerEntity, ver};
use rule::parse_app;
use sea_orm::ActiveValue::Set;
use sea_orm::sqlx::types::chrono::Local;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde_json::json;
use std::sync::Arc;

type SharedStatus = Arc<DashMap<&'static str, Vec<String>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let opt = "sqlite:///C:/Users/sharp/AppData/Local/Programs/checkupdate/ver_tab.db";
        let _ = enable_ansi_support::enable_ansi_support();
    }
    #[cfg(unix)]
    let opt = "sqlite:///Users/sharp/ver_tab.db";

    let now = std::time::SystemTime::now();
    let status = Arc::new(DashMap::from_iter([
        ("success", Vec::new()),
        ("failed", Vec::new()),
    ]));
    let db: DatabaseConnection = Database::connect(opt).await?;

    let a = VerEntity::find_by_id("fzf").one(&db).await?.unwrap();
    let aj: serde_json::Value = json!(a);
    println!("{}\n", serde_json::to_string_pretty(&aj)?);

    #[cfg(windows)]
    let apps = VerEntity::find()
        .filter(
            ver::Column::Platform
                .eq("Windows")
                .or(ver::Column::Platform.is_null()),
        )
        .stream(&db)
        .await?;
    // stream::iter(vec_apps)
    #[cfg(unix)]
    let apps = VerEntity::find()
        .filter(
            ver::Column::Platform
                .ne("Windows")
                .or(ver::Column::Platform.is_null()),
        )
        .stream(&db)
        .await?;
    apps.for_each_concurrent(64, |app| {
        let db = db.clone();
        let status = status.clone();
        async move {
            if let Ok(app) = app {
                let _ = update_app(app, db, status).await;
            }
        }
    })
    .await;
    // apps.map(|app| {
    //         let db = db.clone();
    //         let status = status.clone();
    //         async move {
    //             if let Ok(app)=app{
    //                 let _=update_app(app, db, status).await;
    //             }
    //         }
    //     })
    //     .buffer_unordered(64)
    //     .collect::<Vec<_>>() //for_each
    //     .await;

    println!("用时{:.2?}秒", now.elapsed()?.as_secs_f32());
    print_status(status);
    pause()?;
    Ok(())
}

pub async fn update_app(
    app: ver::Model,
    db: DatabaseConnection,
    status: SharedStatus,
) -> anyhow::Result<()> {
    match parse_app(&app).await {
        Ok(new_ver) if new_ver != app.version => {
            let mut app: ver::ActiveModel = app.into();
            app.version = Set(new_ver.to_owned());
            app.updated_at = Set(Some(Local::now()));
            let app = app.update(&db).await?;
            println!("{} 更新为版本 {}", app.name.green(), new_ver.bright_green());
            status.entry("success").or_default().push(app.name);
            println!("{}", "=".repeat(36));
            Ok(())
        }
        Ok(new_ver) => {
            println!("{} : {}", app.name.bright_cyan(), new_ver.bright_cyan());
            println!("{}", "=".repeat(36));
            Ok(())
        }
        Err(e) => {
            eprintln!("{} 获取版本失败:{}\n{}", app.name, e, "=".repeat(36));
            status.entry("failed").or_default().push(app.name);
            println!("{}", "=".repeat(36));
            Err(e)
        }
    }
}

pub fn print_status(status: SharedStatus) {
    println!(
        "成功: {:?}\n失败: {:?}",
        status
            .get("success")
            .map(|v| v.join(", "))
            .unwrap_or_default(),
        status
            .get("failed")
            .map(|v| v.join(", "))
            .unwrap_or_default()
    );
}
