use futures::StreamExt;
use sea_orm::Database;

use common::{get_db_path, init_status, pause, print_status, query_apps, update_app};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    let _ = enable_ansi_support::enable_ansi_support();

    let now = std::time::Instant::now();
    let status = init_status();
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
