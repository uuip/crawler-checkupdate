use sea_orm::Database;
use tokio::task::JoinSet;

use common::{get_db_path, init_status, pause, print_status, query_apps, update_app};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    let _ = enable_ansi_support::enable_ansi_support();

    let db = Database::connect(get_db_path()).await?;
    let now = std::time::Instant::now();
    let status = init_status();
    let mut apps = query_apps().stream(&db).await?;
    let mut set = JoinSet::new();

    while let Some(Ok(app)) = apps.next().await {
        let db = db.clone();
        let status = status.clone();
        set.spawn(async move { update_app(app, db, &status).await });
    }

    while set.join_next().await.is_some() {}

    println!("用时{:.2}秒", now.elapsed().as_secs_f32());
    print_status(&status);
    pause()?;
    Ok(())
}
