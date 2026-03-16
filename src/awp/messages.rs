use serde::{Deserialize, Serialize};

/// Incoming request from AWP client.
#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub id: String,
    #[serde(rename = "type", default = "default_request_type")]
    pub msg_type: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

fn default_request_type() -> String {
    "request".to_string()
}

/// Outgoing success response.
#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorPayload>,
}

/// Error payload within a response.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// AWP error codes.
pub enum ErrorCode {
    InvalidRequest,
    NotFound,
    Timeout,
    NavigationFailed,
    Internal,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidRequest => "INVALID_REQUEST",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::Timeout => "TIMEOUT",
            ErrorCode::NavigationFailed => "NAVIGATION_FAILED",
            ErrorCode::Internal => "INTERNAL",
        }
    }
}

impl Response {
    pub fn success(id: &str, result: serde_json::Value) -> Self {
        Response {
            id: id.to_string(),
            msg_type: "response".to_string(),
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: &str, code: ErrorCode, message: &str) -> Self {
        Response {
            id: id.to_string(),
            msg_type: "response".to_string(),
            result: None,
            error: Some(ErrorPayload {
                code: code.as_str().to_string(),
                message: message.to_string(),
                details: None,
            }),
        }
    }

    pub fn error_with_details(
        id: &str,
        code: ErrorCode,
        message: &str,
        details: serde_json::Value,
    ) -> Self {
        Response {
            id: id.to_string(),
            msg_type: "response".to_string(),
            result: None,
            error: Some(ErrorPayload {
                code: code.as_str().to_string(),
                message: message.to_string(),
                details: Some(details),
            }),
        }
    }
}
