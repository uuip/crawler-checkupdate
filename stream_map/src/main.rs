use std::collections::HashMap;
use std::io::{stdout, Write};
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

type SharedStatus<'a> = Arc<Mutex<HashMap<&'a str, Vec<&'a str>>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    let opt = "sqlite:///C:/Users/sharp/AppData/Local/Programs/checkupdate/ver_tab.db";
    #[cfg(windows)]
    let _ = enable_ansi_support::enable_ansi_support();
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
    let status = status.lock().unwrap();
    println!(
        "成功: {:?}\n失败: {:?}",
        status.get("success").unwrap().join(", "),
        status.get("failed").unwrap().join(", ")
    );
    pause();
    Ok(())
}

async fn update_app(app: ver::Model, db: DatabaseConnection, status: SharedStatus<'_>) {
    match parse_app(&app).await {
        Ok(new_ver) if new_ver != app.ver => {
            let mut app: ver::ActiveModel = app.into();
            app.ver = Set(new_ver.to_owned());
            app.updated_at = Set(Some(Local::now()));
            let app = app.update(&db).await.unwrap();
            println!("{} 更新为版本 {}", app.name.green(), new_ver.bright_green());
            let mut status = status.lock().unwrap();
            status.get_mut("success").unwrap().push(app.name.leak());
        }
        Ok(new_ver) => {
            println!("{} : {}", app.name.bright_cyan(), new_ver.bright_cyan());
        }
        _ => {
            eprintln!("{} 获取版本失败\n{}", app.name, "=".repeat(36));
            let mut status = status.lock().unwrap();
            status.get_mut("failed").unwrap().push(app.name.leak());
            return;
        }
    }
    println!("{}", "=".repeat(36));
}

fn pause() {
    let mut stdout = stdout();
    stdout.write_all(b"Press any key to continue...").unwrap();
    stdout.flush().unwrap();
    enable_raw_mode().unwrap();
    loop {
        if let Event::Key(_) = event::read().unwrap() {
            break;
        }
    }
    disable_raw_mode().unwrap();
}
