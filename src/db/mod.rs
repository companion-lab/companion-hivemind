use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
use crate::config::Settings;

pub async fn connect(settings: &Settings) -> anyhow::Result<PgPool> {
    let schema = settings.normalized_db_schema();
    let pool = PgPoolOptions::new()
        .max_connections(settings.db_max_connections)
        .after_connect({
            let schema = schema.clone();
            move |conn, _meta| {
                let schema = schema.clone();
                Box::pin(async move {
                    // Ensure schema exists before setting search_path
                    let create_schema = format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema.replace('"', "\"\""));
                    conn.execute(create_schema.as_str()).await?;
                    let query = format!("SET search_path TO \"{}\"", schema.replace('"', "\"\""));
                    conn.execute(query.as_str()).await?;
                    Ok(())
                })
            }
        })
        .connect_with(
            sqlx::postgres::PgConnectOptions::new()
                .host(&settings.db_host)
                .port(settings.db_port)
                .database(&settings.db_name)
                .username(&settings.db_user)
                .password(&settings.db_password)
        )
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!(schema = %schema, "Database connected and migrated");
    Ok(pool)
}
