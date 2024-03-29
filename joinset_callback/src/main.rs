#![allow(dead_code, unused_variables)]

use std::collections::HashMap;

use anyhow::Error;
use mincolor::*;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait};
use serde_json::json;
use tokio::task::JoinSet;

use models::ver;
use models::VerEntity;
use rule::{num_version, parse_app};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt: &str = if cfg!(target_os = "windows") {
        let _ = enable_ansi_support::enable_ansi_support();
        //"postgres://postgres:postgres@127.0.0.1/postgres"
        "sqlite:///C:/Users/sharp/AppData/Local/Programs/checkupdate/ver_tab.db"
    } else {
        "sqlite:///Users/sharp/Downloads/ver_tab.db"
    };
    let now = std::time::SystemTime::now();
    let mut status: HashMap<&str, Vec<&str>> =
        HashMap::from([("success", Vec::new()), ("failed", Vec::new())]);

    let db: DatabaseConnection = Database::connect(opt).await?;
    let a = VerEntity::find_by_id("fzf").one(&db).await?.unwrap();
    let aj: serde_json::Value = json!(a);
    println!("{}\n", serde_json::to_string_pretty(&aj)?);

    let apps = VerEntity::find().all(&db).await?;
    let mut set: JoinSet<Result<(ver::Model, Option<String>), Error>> = JoinSet::new();
    for app in apps {
        let msg: String = format!("{} 提取版本号失败", app.name);
        set.spawn(
            async move { Ok((app.clone(), parse_app(&app).await.map_or(None, num_version))) },
        );
    }

    while let Some(res) = set.join_next().await {
        let (app, new_ver) = match res? {
            Ok(r) => r,
            _ => continue,
        };
        update_app(app, &db, new_ver, &mut status).await;
    }

    println!("用时{:.2?}秒", now.elapsed()?.as_secs_f32());
    println!(
        "成功: {:?}\n失败: {:?}",
        status.get("success").unwrap().join(", "),
        status.get("failed").unwrap().join(", ")
    );
    if cfg!(target_os = "windows") & !cfg!(debug_assertions) {
        let _ = std::process::Command::new("cmd.exe")
            .arg("/c")
            .arg("pause")
            .status();
    }
    Ok(())
}

async fn update_app(
    app: ver::Model,
    db: &DatabaseConnection,
    new_ver: Option<String>,
    status: &mut HashMap<&str, Vec<&str>>,
) {
    let new_ver = if let Some(s) = new_ver {
        s
    } else {
        eprintln!("{} 获取版本失败\n{}", app.name, "=".repeat(36));
        status
            .get_mut("failed")
            .unwrap()
            .push(Box::leak(app.name.into_boxed_str()));
        return;
    };
    if new_ver != app.ver {
        let mut app: ver::ActiveModel = app.into();
        app.ver = Set(new_ver.to_owned());
        let app = app.update(db).await.unwrap();
        println!("{} 更新为版本 {}", app.name.green(), new_ver.bright_green());
        status
            .get_mut("success")
            .unwrap()
            .push(Box::leak(app.name.into_boxed_str()));
    } else {
        println!("{} : {}", app.name.bright_cyan(), new_ver.bright_cyan());
    }
    println!("{}", "=".repeat(36));
}
