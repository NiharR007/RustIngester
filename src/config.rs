use std::env;

#[derive(Clone)]
pub struct Config {
    pub db_url: String,
    pub lsh_buckets: usize,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let lsh_buckets = env::var("LSH_BUCKETS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(128);
        Self { db_url, lsh_buckets }
    }
}
