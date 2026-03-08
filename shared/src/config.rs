use std::env;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_hours: i64,
    pub refresh_expiry_hours: i64,
}

#[derive(Debug, Clone)]
pub struct NatsConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct MongoConfig {
    pub url: String,
    pub database: String,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }
}

impl JwtConfig {
    pub fn from_env() -> Self {
        Self {
            secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            refresh_expiry_hours: env::var("JWT_REFRESH_EXPIRY_HOURS")
                .unwrap_or_else(|_| "168".to_string())
                .parse()
                .unwrap_or(168),
        }
    }
}

impl NatsConfig {
    pub fn from_env() -> Self {
        Self {
            url: env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),
        }
    }
}

impl MongoConfig {
    pub fn from_env() -> Self {
        Self {
            url: env::var("MONGO_URL").unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            database: env::var("MONGO_DATABASE").unwrap_or_else(|_| "notifications".to_string()),
        }
    }
}

impl RedisConfig {
    pub fn from_env() -> Self {
        Self {
            url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        }
    }
}
