use anyhow::Result;
use colored::*;
use futures::StreamExt;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection};
use std::collections::HashMap;
use tokio::sync::mpsc;

use common::{FAILED_KEY, SEPARATOR, SUCCESS_KEY, get_db_path, pause, print_status, query_apps};
use models::ver;
use rule::parse_app;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(windows)]
    let _ = enable_ansi_support::enable_ansi_support();

    let db = Database::connect(get_db_path()).await?;
    let now = std::time::Instant::now();
    let mut status = HashMap::from([(SUCCESS_KEY, Vec::new()), (FAILED_KEY, Vec::new())]);
    let (tx, mut rx) = mpsc::channel(100);
    let mut apps = query_apps().stream(&db).await?;
    while let Some(Ok(app)) = apps.next().await {
        let tx = tx.clone();
        tokio::spawn(async move {
            let new_ver = parse_app(&app).await;
            let _ = tx.send((app, new_ver)).await;
        });
    }

    drop(tx);

    while let Some((app, new_ver)) = rx.recv().await {
        update_app(app, &db, new_ver, &mut status).await;
    }

    println!("用时{:.2}秒", now.elapsed().as_secs_f32());
    print_status(&status);
    pause()?;
    Ok(())
}

async fn update_app(
    app: ver::Model,
    db: &DatabaseConnection,
    new_ver: Result<String>,
    status: &mut HashMap<&str, Vec<String>>,
) {
    let app_name = app.name.clone();

    match new_ver {
        Ok(new_ver) if new_ver != app.version => {
            let mut active_model: ver::ActiveModel = app.into();
            active_model.version = Set(new_ver.clone());
            if active_model.update(db).await.is_ok() {
                println!("{} 更新为版本 {}", app_name.green(), new_ver.bright_green());
                status.entry(SUCCESS_KEY).or_default().push(app_name);
            }
        }
        Ok(new_ver) => {
            println!("{} : {}", app_name.bright_cyan(), new_ver.bright_cyan());
        }
        Err(e) => {
            eprintln!("{} 获取版本失败:{}", app_name, e);
            status.entry(FAILED_KEY).or_default().push(app_name);
        }
    }
    println!("{SEPARATOR}");
}
