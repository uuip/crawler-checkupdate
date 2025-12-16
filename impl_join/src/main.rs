use sea_orm::Database;
use tokio::task;

use common::{get_db_path, init_status, pause, print_status, query_apps, update_app};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    let _ = enable_ansi_support::enable_ansi_support();

    let db = Database::connect(get_db_path()).await?;
    let now = std::time::Instant::now();
    let status = init_status();
    let apps = query_apps().all(&db).await?;
    let tasks: Vec<_> = apps
        .into_iter()
        .map(|app| {
            let db = db.clone();
            let status = status.clone();
            task::spawn(async move { update_app(app, db, &status).await })
        })
        .collect();
    futures::future::join_all(tasks).await;

    println!("用时{:.2}秒", now.elapsed().as_secs_f32());
    print_status(&status);
    pause()?;
    Ok(())
}
