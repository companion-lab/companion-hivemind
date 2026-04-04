use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_db_host")]
    pub db_host: String,
    #[serde(default = "default_db_port")]
    pub db_port: u16,
    #[serde(default = "default_db_name")]
    pub db_name: String,
    #[serde(default = "default_db_user")]
    pub db_user: String,
    #[serde(default = "default_db_password")]
    pub db_password: String,
    #[serde(default = "default_db_max_connections")]
    pub db_max_connections: u32,
    #[serde(default = "default_db_schema")]
    pub db_schema: String,

    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_ttl_seconds")]
    pub jwt_ttl_seconds: i64,

    #[serde(default = "default_encryption_secret")]
    pub encryption_secret: String,

    #[serde(default = "default_vexa_api_url")]
    pub vexa_api_url: String,
    #[serde(default = "default_vexa_admin_api_url")]
    pub vexa_admin_api_url: String,
    #[serde(default)]
    pub vexa_admin_token: String,

    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,

    // ─── Embeddings (Ollama) ──────────────────────────────
    #[serde(default = "default_embedding_api_url")]
    pub embedding_api_url: String,
    #[serde(default)]
    pub embedding_api_key: String,
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,

    // ─── Qdrant ────────────────────────────────────────────
    #[serde(default = "default_qdrant_url")]
    pub qdrant_url: String,
    #[serde(default)]
    pub qdrant_api_key: String,
}

// Default functions for serde
fn default_db_host() -> String { "supabase-db".into() }
fn default_db_port() -> u16 { 5432 }
fn default_db_name() -> String { "postgres".into() }
fn default_db_user() -> String { "postgres".into() }
fn default_db_password() -> String { "postgres".into() }
fn default_db_max_connections() -> u32 { 10 }
fn default_db_schema() -> String { "hivemind".into() }
fn default_jwt_secret() -> String { "hivemind-secret-change-me".into() }
fn default_jwt_ttl_seconds() -> i64 { 30 * 24 * 60 * 60 }
fn default_encryption_secret() -> String { "hivemind-encryption-secret-change-me".into() }
fn default_vexa_api_url() -> String { "http://vexa-api-gateway:8000".into() }
fn default_vexa_admin_api_url() -> String { "http://vexa-admin-api:8001".into() }
fn default_host() -> String { "0.0.0.0".into() }
fn default_port() -> u16 { 9100 }
fn default_embedding_api_url() -> String { "http://localhost:11434".into() }
fn default_embedding_model() -> String { "nomic-embed-text".into() }
fn default_qdrant_url() -> String { "http://localhost:6334".into() }

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
            db_host: default_db_host(),
            db_port: default_db_port(),
            db_name: default_db_name(),
            db_user: default_db_user(),
            db_password: default_db_password(),
            db_max_connections: default_db_max_connections(),
            db_schema: default_db_schema(),

            jwt_secret: default_jwt_secret(),
            jwt_ttl_seconds: default_jwt_ttl_seconds(),

            encryption_secret: default_encryption_secret(),

            vexa_api_url: default_vexa_api_url(),
            vexa_admin_api_url: default_vexa_admin_api_url(),
            vexa_admin_token: String::new(),

            host: default_host(),
            port: default_port(),

            embedding_api_url: default_embedding_api_url(),
            embedding_api_key: String::new(),
            embedding_model: default_embedding_model(),

            qdrant_url: default_qdrant_url(),
            qdrant_api_key: String::new(),
        }
    }
}

pub fn load() -> Settings {
    dotenvy::dotenv().ok();
    let result = config::Config::builder()
        .add_source(config::Environment::default().separator("__"))
        .build();
    
    match result {
        Ok(c) => {
            match c.try_deserialize::<Settings>() {
                Ok(settings) => settings,
                Err(e) => {
                    eprintln!("Failed to deserialize config: {}", e);
                    Settings::default()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to build config: {}", e);
            Settings::default()
        }
    }
}
