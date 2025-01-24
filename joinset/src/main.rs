use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mincolor::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait};
use tokio::task::JoinSet;

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
    let db: DatabaseConnection = Database::connect(opt).await?;

    let now = std::time::SystemTime::now();
    let status: SharedStatus = Arc::new(Mutex::new(HashMap::from([
        ("success", Vec::new()),
        ("failed", Vec::new()),
    ])));

    let apps: Vec<ver::Model> = VerEntity::find().all(&db).await?;
    let mut set = JoinSet::new();
    for app in apps {
        let db = db.clone();
        let status = status.clone();
        set.spawn(async move { update_app(app, db, status).await });
    }

    while set.join_next().await.is_some() {}

    println!("用时{:.2?}秒", now.elapsed()?.as_secs_f32());
    let status = status.lock().unwrap();
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
            let app = app.update(&db).await?;
            println!("{} 更新为版本 {}", app.name.green(), new_ver.bright_green());
            let mut status = status.lock().unwrap();
            status.get_mut("success").unwrap().push(app.name.leak());
        }
        Ok(new_ver) => println!("{} : {}", app.name.bright_cyan(), new_ver.bright_cyan()),
        Err(e) => {
            eprintln!("{} 获取版本失败:{}\n{}", app.name, e, "=".repeat(36));
            let mut status = status.lock().unwrap();
            status.get_mut("failed").unwrap().push(app.name.leak());
        }
    }
    println!("{}", "=".repeat(36));
    Ok(())
}
