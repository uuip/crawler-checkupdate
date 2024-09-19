use anyhow::Result;
use mincolor::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use models::ver;
use models::VerEntity;
use rule::parse_app;

#[tokio::main]
async fn main() -> Result<()> {
    let opt: &str = if cfg!(target_os = "windows") {
        let _ = enable_ansi_support::enable_ansi_support();
        //"postgres://postgres:postgres@127.0.0.1/postgres"
        "sqlite:///C:/Users/sharp/AppData/Local/Programs/checkupdate/ver_tab.db"
    } else {
        "sqlite:///Users/sharp/Downloads/ver_tab.db"
    };
    let db: DatabaseConnection = Database::connect(opt).await?;

    let now = std::time::SystemTime::now();
    let mut status: HashMap<&str, Vec<&str>> =
        HashMap::from([("success", Vec::new()), ("failed", Vec::new())]);

    let apps: Vec<ver::Model> = VerEntity::find().all(&db).await?;

    let (tx, mut rx) = mpsc::channel(100);
    let mut set = JoinSet::new();

    for app in apps {
        let tx = tx.clone();
        set.spawn(async move {
            let new_ver = parse_app(&app).await;
            let _ = tx.send((app, new_ver)).await;
        });
    }

    drop(tx);

    while let Some(i) = rx.recv().await {
        let (app, new_ver): (ver::Model, Result<String>) = i;
        update_app(app, &db, new_ver, &mut status).await;
    }

    println!("用时{:.2?}秒", now.elapsed()?.as_secs_f32());
    println!(
        "成功: {:?}\n失败: {:?}",
        status.get("success").unwrap().join(", "),
        status.get("failed").unwrap().join(", ")
    );
    Ok(())
}

async fn update_app(
    app: ver::Model,
    db: &DatabaseConnection,
    new_ver: Result<String>,
    status: &mut HashMap<&str, Vec<&str>>,
) {
    match new_ver {
        Ok(new_ver) if new_ver != app.ver => {
            let mut app: ver::ActiveModel = app.into();
            app.ver = Set(new_ver.to_owned());
            let app = app.update(db).await.unwrap();
            println!("{} 更新为版本 {}", app.name.green(), new_ver.bright_green());
            status
                .get_mut("success")
                .unwrap()
                .push(Box::leak(app.name.into_boxed_str()));
        }
        Ok(new_ver) => println!("{} : {}", app.name.bright_cyan(), new_ver.bright_cyan()),
        Err(e) => {
            eprintln!("{} 获取版本失败:{}\n{}", app.name, e, "=".repeat(36));
            status
                .get_mut("failed")
                .unwrap()
                .push(Box::leak(app.name.into_boxed_str()));
            return;
        }
    }
    println!("{}", "=".repeat(36));
}
