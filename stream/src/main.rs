use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::json;

use common::{pause, print_status, update_app, SharedStatus};
use models::ver;
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
        .stream(&db)
        .await?;
    // stream::iter(vec_apps)
    #[cfg(unix)]
    let apps = VerEntity::find()
        .filter(
            ver::Column::Platform
                .ne("Windows")
                .or(ver::Column::Platform.is_null()),
        )
        .stream(&db)
        .await?;
    apps.for_each_concurrent(64, |app| {
        let db = db.clone();
        let status = status.clone();
        async move {
            if let Ok(app) = app {
                let _ = update_app(app, db, status).await;
            }
        }
    })
    .await;
    // apps.map(|app| {
    //         let db = db.clone();
    //         let status = status.clone();
    //         async move {
    //             if let Ok(app)=app{
    //                 let _=update_app(app, db, status).await;
    //             }
    //         }
    //     })
    //     .buffer_unordered(64)
    //     .collect::<Vec<_>>() //for_each
    //     .await;

    println!("用时{:.2?}秒", now.elapsed()?.as_secs_f32());
    print_status(status);
    pause()?;
    Ok(())
}
