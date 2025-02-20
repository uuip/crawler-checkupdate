use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sea_orm::{Database, DatabaseConnection, EntityTrait};
use tokio::task;

use common::{SharedStatus, pause, print_status, update_app};
use models::VerEntity;
use models::ver;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let opt = "sqlite:///C:/Users/sharp/AppData/Local/Programs/checkupdate/ver_tab.db";
        let _ = enable_ansi_support::enable_ansi_support();
    }
    #[cfg(unix)]
    let opt = "sqlite:///Users/sharp/ver_tab.db";
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
    print_status(status);
    pause()?;
    Ok(())
}
