use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

use chrono::offset::Local;
use colored::*;
use crossterm::event;
use crossterm::event::Event;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use models::ver;
use rule::parse_app;
use sea_orm::entity::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

pub type SharedStatus = Arc<Mutex<HashMap<&'static str, Vec<String>>>>;

pub fn pause() -> std::io::Result<()> {
    println!("Press any key to continue...");
    std::io::stdout().flush()?;
    enable_raw_mode()?;
    while !matches!(event::read(), Ok(Event::Key(_))) {}
    disable_raw_mode()
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
            {
                let mut status = status.lock().unwrap();
                status.entry("success").or_default().push(app.name);
            }
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
            {
                let mut status = status.lock().unwrap();
                status.entry("failed").or_default().push(app.name);
            }
            println!("{}", "=".repeat(36));
            Err(e)
        }
    }
}

pub fn print_status(status: SharedStatus) {
    let status = status.lock().unwrap();
    println!(
        "成功: {:?}\n失败: {:?}",
        status
            .get("success")
            .cloned()
            .map(|v| v.join(", "))
            .unwrap_or_default(),
        status
            .get("failed")
            .cloned()
            .map(|v| v.join(", "))
            .unwrap_or_default()
    );
}
