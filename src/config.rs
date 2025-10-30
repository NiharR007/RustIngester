use std::env;

#[derive(Clone)]
pub struct Config {
    pub db_url: String,
    pub lsh_buckets: usize,
    pub embed_model_path: Option<String>,
    pub embed_server_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let lsh_buckets = env::var("LSH_BUCKETS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(128);
        let embed_model_path = env::var("EMBED_MODEL_PATH").ok();
        let embed_server_url = env::var("EMBED_SERVER_URL").ok();
        
        // Log configuration on startup
        eprintln!("📋 Configuration loaded:");
        eprintln!("   DATABASE_URL: {}", if db_url.is_empty() { "NOT SET" } else { "SET" });
        eprintln!("   LSH_BUCKETS: {}", lsh_buckets);
        eprintln!("   EMBED_MODEL_PATH: {}", embed_model_path.as_deref().unwrap_or("NOT SET"));
        eprintln!("   EMBED_SERVER_URL: {}", embed_server_url.as_deref().unwrap_or("NOT SET"));
        
        Self { db_url, lsh_buckets, embed_model_path, embed_server_url }
    }
}
