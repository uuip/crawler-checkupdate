use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

use colored::*;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{event, event::Event};
use futures_util::{stream, StreamExt};
use sea_orm::sqlx::types::chrono::Local;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde_json::json;

use models::ver;
use models::VerEntity;
use rule::parse_app;

type SharedStatus<'a> = Arc<Mutex<HashMap<&'static str, Vec<String>>>>;

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
    let status: SharedStatus = Arc::new(Mutex::new(HashMap::from([
        ("success", Vec::new()),
        ("failed", Vec::new()),
    ])));

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
        .all(&db)
        .await?;
    #[cfg(unix)]
    let apps = VerEntity::find()
        .filter(
            ver::Column::Platform
                .ne("Windows")
                .or(ver::Column::Platform.is_null()),
        )
        .all(&db)
        .await?;
    stream::iter(apps)
        .map(|app| {
            let db = db.clone();
            let status = status.clone();
            async move { update_app(app, db, status).await }
        })
        .buffer_unordered(64)
        .collect::<Vec<_>>()
        .await;

    println!("用时{:.2?}秒", now.elapsed()?.as_secs_f32());
    {
        let status = status.lock().unwrap();
        println!(
            "成功: {:?}\n失败: {:?}",
            status
                .get("success")
                .cloned()
                .map(|item| item.join(", "))
                .unwrap_or_default(),
            status
                .get("failed")
                .cloned()
                .map(|item| item.join(", "))
                .unwrap_or_default()
        );
    }
    pause()?;
    Ok(())
}

async fn update_app(
    app: ver::Model,
    db: DatabaseConnection,
    status: SharedStatus<'static>,
) -> anyhow::Result<()> {
    match parse_app(&app).await {
        Ok(new_ver) if new_ver != app.verion => {
            let mut app: ver::ActiveModel = app.into();
            app.verion = Set(new_ver.to_owned());
            app.updated_at = Set(Some(Local::now()));
            let app = app.update(&db).await?;
            println!("{} 更新为版本 {}", app.name.green(), new_ver.bright_green());
            {
                let mut status = status.lock().unwrap();
                status.entry("success").or_default().push(app.name);
            }
        }
        Ok(new_ver) => {
            println!("{} : {}", app.name.bright_cyan(), new_ver.bright_cyan());
        }
        Err(e) => {
            eprintln!("{} 获取版本失败:{}\n{}", app.name, e, "=".repeat(36));
            {
                let mut status = status.lock().unwrap();
                status.entry("failed").or_default().push(app.name);
            }
            return Err(e);
        }
    }
    println!("{}", "=".repeat(36));
    Ok(())
}

fn pause() -> std::io::Result<()> {
    println!("Press any key to continue...");
    std::io::stdout().flush()?;
    enable_raw_mode()?;
    while !matches!(event::read(), Ok(Event::Key(_))) {}
    disable_raw_mode()
}
