use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mincolor::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait};
use tokio::task;

use models::ver;
use models::VerEntity;
use rule::parse_app;

type SharedStatus<'a> = Arc<Mutex<HashMap<&'a str, Vec<&'a str>>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt: &str = if cfg!(target_os = "windows") {
        let _ = enable_ansi_support::enable_ansi_support();
        "sqlite:///C:/Users/sharp/AppData/Local/Programs/checkupdate/ver_tab.db"
    } else {
        "sqlite:///Users/sharp/ver_tab.db"
    };
    let db: DatabaseConnection = Database::connect(opt).await?;

    let now = std::time::SystemTime::now();
    let status: SharedStatus = Arc::new(Mutex::new(HashMap::from([
        ("success", Vec::new()),
        ("failed", Vec::new()),
    ])));

    let apps: Vec<ver::Model> = VerEntity::find().all(&db).await?;
    let tasks: Vec<_> = apps
        .into_iter()
        .map(|app| {
            let db = db.clone();
            let status = status.clone();
            task::spawn(async move { update_app(app, db, status).await })
        })
        .collect();
    futures::future::join_all(tasks).await;

    println!("用时{:.2?}秒", now.elapsed()?.as_secs_f32());

    let status = status.lock().unwrap();
    println!(
        "成功: {:?}\n失败: {:?}",
        status.get("success").unwrap().join(", "),
        status.get("failed").unwrap().join(", ")
    );
    Ok(())
}

async fn update_app(app: ver::Model, db: DatabaseConnection, status: SharedStatus<'static>) {
    match parse_app(&app).await {
        Ok(new_ver) if new_ver != app.ver => {
            let mut app: ver::ActiveModel = app.into();
            app.ver = Set(new_ver.to_owned());
            let app = app.update(&db).await.unwrap();
            println!("{} 更新为版本 {}", app.name.green(), new_ver.bright_green());
            let mut status = status.lock().unwrap();
            status
                .get_mut("success")
                .unwrap()
                .push(Box::leak(app.name.into_boxed_str()));
        }
        Ok(new_ver) => {
            println!("{} : {}", app.name.bright_cyan(), new_ver.bright_cyan());
        }
        Err(e) => {
            eprintln!("{} 获取版本失败:{}\n{}", app.name, e, "=".repeat(36));
            let mut status = status.lock().unwrap();
            status
                .get_mut("failed")
                .unwrap()
                .push(Box::leak(app.name.into_boxed_str()));
            return;
        }
    }

    println!("{}", "=".repeat(36));
}
