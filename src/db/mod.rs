use sqlx::PgPool;
use crate::config::Settings;

pub async fn connect(settings: &Settings) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect_with(
        sqlx::postgres::PgConnectOptions::new()
            .host(&settings.db_host)
            .port(settings.db_port)
            .database(&settings.db_name)
            .username(&settings.db_user)
            .password(&settings.db_password)
    )
    .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database connected and migrated");
    Ok(pool)
}
