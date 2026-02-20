use thiserror::Error;

#[derive(Error, Debug)]
pub enum GamploError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Missing field in response: {field}, response: {response}")]
    MissingField { field: String, response: String },

    #[error("Failed to deserialize {type_name}: {source}, data: {data}")]
    Deserialization {
        type_name: String,
        data: String,
        source: serde_json::Error,
    },

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Token not found in environment variables or query parameters")]
    TokenNotFound(String),

    #[cfg(target_arch = "wasm32")]
    #[error("WASM error: {0}")]
    Wasm(String),
}