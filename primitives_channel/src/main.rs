use anyhow::Result;
use colored::*;
use futures::StreamExt;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait};
use std::collections::HashMap;
use tokio::sync::mpsc;

use common::pause;
use models::ver;
use models::VerEntity;
use rule::parse_app;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(windows)]
    {
        let opt = "sqlite:///C:/Users/sharp/AppData/Local/Programs/checkupdate/ver_tab.db";
        let _ = enable_ansi_support::enable_ansi_support();
    }
    #[cfg(unix)]
    let opt = "sqlite:///Users/sharp/ver_tab.db";

    let db: DatabaseConnection = Database::connect(opt).await?;
    let now = std::time::SystemTime::now();
    let mut status: HashMap<&str, Vec<String>> =
        HashMap::from([("success", Vec::new()), ("failed", Vec::new())]);
    let (tx, mut rx) = mpsc::channel(100);

    let mut apps = VerEntity::find().stream(&db).await?;
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

    println!("用时{:.2?}秒", now.elapsed()?.as_secs_f32());
    println!(
        "成功: {:?}\n失败: {:?}",
        status
            .get("success")
            .map(|item| item.join(", "))
            .unwrap_or_default(),
        status
            .get("failed")
            .map(|item| item.join(", "))
            .unwrap_or_default()
    );
    pause()?;
    Ok(())
}

async fn update_app(
    app: ver::Model,
    db: &DatabaseConnection,
    new_ver: Result<String>,
    status: &mut HashMap<&str, Vec<String>>,
) {
    match new_ver {
        Ok(new_ver) if new_ver != app.version => {
            let mut app: ver::ActiveModel = app.into();
            app.version = Set(new_ver.to_owned());
            let app = app.update(db).await.unwrap();
            println!("{} 更新为版本 {}", app.name.green(), new_ver.bright_green());
            status.entry("success").or_default().push(app.name);
        }
        Ok(new_ver) => println!("{} : {}", app.name.bright_cyan(), new_ver.bright_cyan()),
        Err(e) => {
            eprintln!("{} 获取版本失败:{}\n{}", app.name, e, "=".repeat(36));
            status.entry("failed").or_default().push(app.name);
            return;
        }
    }
    println!("{}", "=".repeat(36));
}
