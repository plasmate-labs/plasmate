//! CDP protocol types.

use serde::{Deserialize, Serialize};

/// Incoming CDP request.
#[derive(Debug, Clone, Deserialize)]
pub struct CdpRequest {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    /// Session ID for target-attached messages.
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

/// Outgoing CDP response.
#[derive(Debug, Clone, Serialize)]
pub struct CdpResponse {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CdpError>,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// CDP error object.
#[derive(Debug, Clone, Serialize)]
pub struct CdpError {
    pub code: i64,
    pub message: String,
}

/// CDP event (server-initiated).
#[derive(Debug, Clone, Serialize)]
pub struct CdpEvent {
    pub method: String,
    pub params: serde_json::Value,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl CdpResponse {
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        CdpResponse {
            id,
            result: Some(result),
            error: None,
            session_id: None,
        }
    }

    pub fn error(id: u64, code: i64, message: &str) -> Self {
        CdpResponse {
            id,
            result: None,
            error: Some(CdpError {
                code,
                message: message.to_string(),
            }),
            session_id: None,
        }
    }

    pub fn with_session(mut self, session_id: Option<String>) -> Self {
        self.session_id = session_id;
        self
    }
}

impl CdpEvent {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        CdpEvent {
            method: method.to_string(),
            params,
            session_id: None,
        }
    }
}

/// CDP error codes (from Chrome spec).
pub const CDP_ERR_SERVER: i64 = -32000;
pub const CDP_ERR_NOT_FOUND: i64 = -32601;
pub const CDP_ERR_INVALID_PARAMS: i64 = -32602;
