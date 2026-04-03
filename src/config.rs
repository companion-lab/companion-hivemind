use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub db_max_connections: u32,
    pub db_schema: String,

    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,

    pub encryption_secret: String,

    pub vexa_api_url: String,
    pub vexa_admin_api_url: String,
    pub vexa_admin_token: String,

    pub host: String,
    pub port: u16,

    // ─── Embeddings (OpenRouter / OpenAI-compatible) ───────
    pub embedding_api_url: String,
    pub embedding_api_key: String,
    pub embedding_model: String,

    // ─── Qdrant ────────────────────────────────────────────
    pub qdrant_url: String,
    pub qdrant_api_key: String,
}

impl Settings {
    pub fn normalized_db_schema(&self) -> String {
        let schema = self.db_schema.trim();
        if schema.is_empty() {
            "hivemind".into()
        } else {
            schema.into()
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            db_host: "supabase-db".into(),
            db_port: 5432,
            db_name: "postgres".into(),
            db_user: "postgres".into(),
            db_password: "postgres".into(),
            db_max_connections: 10,
            db_schema: "hivemind".into(),

            jwt_secret: "hivemind-secret-change-me".into(),
            jwt_ttl_seconds: 30 * 24 * 60 * 60,

            encryption_secret: "hivemind-encryption-secret-change-me".into(),

            vexa_api_url: "http://vexa-api-gateway:8000".into(),
            vexa_admin_api_url: "http://vexa-admin-api:8001".into(),
            vexa_admin_token: String::new(),

            host: "0.0.0.0".into(),
            port: 9100,

            embedding_api_url: "https://openrouter.ai/api/v1/embeddings".into(),
            embedding_api_key: String::new(),
            embedding_model: "openai/text-embedding-3-small".into(),

            qdrant_url: "http://localhost:6334".into(),
            qdrant_api_key: String::new(),
        }
    }
}

pub fn load() -> Settings {
    dotenvy::dotenv().ok();
    config::Config::builder()
        .add_source(config::Environment::default().separator("__"))
        .build()
        .ok()
        .and_then(|c| c.try_deserialize().ok())
        .unwrap_or_default()
}
