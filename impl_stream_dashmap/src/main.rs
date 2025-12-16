use common::{get_db_path, pause, print_status, query_apps, update_app};
use dashmap::DashMap;
use futures::StreamExt;
use sea_orm::Database;
use std::sync::Arc;

type SharedStatus = Arc<DashMap<&'static str, Vec<String>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    let _ = enable_ansi_support::enable_ansi_support();

    let now = std::time::Instant::now();
    let status: SharedStatus = Arc::new(DashMap::from_iter([
        ("success", Vec::new()),
        ("failed", Vec::new()),
    ]));
    let db = Database::connect(get_db_path()).await?;
    let apps = query_apps().stream(&db).await?;

    apps.for_each_concurrent(64, |app| {
        let db = db.clone();
        let status = status.clone();
        async move {
            if let Ok(app) = app {
                let _ = update_app(app, db, &status).await;
            }
        }
    })
    .await;

    println!("用时{:.2}秒", now.elapsed().as_secs_f32());
    print_status(&status);
    pause()?;
    Ok(())
}
