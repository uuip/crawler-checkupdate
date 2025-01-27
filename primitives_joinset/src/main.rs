use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sea_orm::{Database, DatabaseConnection, EntityTrait};
use tokio::task::JoinSet;

use common::{pause, print_status, update_app, SharedStatus};
use futures::StreamExt;
use models::VerEntity;

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

    let mut apps = VerEntity::find().stream(&db).await?;
    let mut set = JoinSet::new();
    while let Some(Ok(app)) = apps.next().await {
        let db = db.clone();
        let status = status.clone();
        set.spawn(async move { update_app(app, db, status).await });
    }

    while set.join_next().await.is_some() {}

    println!("用时{:.2?}秒", now.elapsed()?.as_secs_f32());
    print_status(status);
    pause()?;
    Ok(())
}
