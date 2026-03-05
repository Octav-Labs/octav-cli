use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum OctavError {
    #[error("Invalid API key. Please check your OCTAV_API_KEY.")]
    Auth(String),

    #[error("Insufficient credits: {0}")]
    InsufficientCredits(String, Option<f64>),

    #[error("Rate limit exceeded")]
    RateLimit(String, Option<u64>),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("No API key configured. Use `octav auth set-key <KEY>`, set OCTAV_API_KEY, or pass --api-key.")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),
}

impl OctavError {
    pub fn to_json(&self) -> Value {
        match self {
            OctavError::Auth(msg) => json!({
                "error": { "type": "auth", "message": msg, "status": 401 }
            }),
            OctavError::InsufficientCredits(msg, credits) => {
                let mut err = json!({
                    "error": { "type": "insufficient_credits", "message": msg, "status": 402 }
                });
                if let Some(c) = credits {
                    err["error"]["credits_needed"] = json!(c);
                }
                err
            }
            OctavError::RateLimit(msg, retry) => {
                let mut err = json!({
                    "error": { "type": "rate_limit", "message": msg, "status": 429 }
                });
                if let Some(r) = retry {
                    err["error"]["retry_after"] = json!(r);
                }
                err
            }
            OctavError::Api { status, message } => json!({
                "error": { "type": "api", "message": message, "status": status }
            }),
            OctavError::Validation(msg) => json!({
                "error": { "type": "validation", "message": msg }
            }),
            OctavError::Config(msg) => json!({
                "error": { "type": "config", "message": msg }
            }),
            OctavError::Network(msg) => json!({
                "error": { "type": "network", "message": msg }
            }),
        }
    }
}
