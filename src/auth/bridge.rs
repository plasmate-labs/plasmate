//! Local HTTP bridge server for Chrome extension integration.
//!
//! Provides endpoints for the Plasmate extension to push cookies directly
//! instead of using clipboard copy.

use crate::auth::store::{self, CookieEntry, CookieProfile};
use axum::{
    extract::Json,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

/// Default port for the bridge server
pub const DEFAULT_PORT: u16 = 9271;

/// Request body for POST /api/cookies
#[derive(Debug, Deserialize)]
pub struct CookiesRequest {
    pub domain: String,
    pub cookies: HashMap<String, CookieValue>,
    #[serde(default)]
    pub expiry: HashMap<String, i64>,
}

/// Cookie value - can be a simple string or an object with expiry
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CookieValue {
    Simple(String),
    WithExpiry {
        value: String,
        #[serde(rename = "expirationDate")]
        expiration_date: Option<f64>,
    },
}

impl CookieValue {
    pub fn into_entry(self, expiry_override: Option<i64>) -> CookieEntry {
        match self {
            CookieValue::Simple(value) => CookieEntry::with_expiry(value, expiry_override),
            CookieValue::WithExpiry {
                value,
                expiration_date,
            } => {
                // Prefer explicit expiry override, then embedded expirationDate
                let exp = expiry_override.or(expiration_date.map(|ts| ts as i64));
                CookieEntry::with_expiry(value, exp)
            }
        }
    }
}

/// Response body for GET /api/status
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub ok: bool,
    pub version: String,
    pub profiles: Vec<String>,
}

/// Response body for POST /api/cookies
#[derive(Debug, Serialize)]
pub struct CookiesResponse {
    pub ok: bool,
    pub domain: String,
    pub cookies_stored: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Start the bridge HTTP server.
pub async fn start(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // CORS layer for chrome-extension:// origins
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([axum::http::header::CONTENT_TYPE])
        .max_age(std::time::Duration::from_secs(86400));

    let app = Router::new()
        .route("/api/status", get(handle_status))
        .route("/api/cookies", post(handle_cookies))
        .layer(cors);

    let listener = TcpListener::bind(addr).await?;
    info!(port = %port, "Auth bridge server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Handle GET /api/status
async fn handle_status() -> impl IntoResponse {
    let profiles = store::list_profiles().unwrap_or_default();
    let response = StatusResponse {
        ok: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        profiles,
    };

    (StatusCode::OK, Json(response))
}

/// Handle POST /api/cookies
async fn handle_cookies(
    Json(request): Json<CookiesRequest>,
) -> impl IntoResponse {
    // Convert cookies, applying expiry from the separate expiry map if provided
    let cookies: HashMap<String, CookieEntry> = request
        .cookies
        .into_iter()
        .map(|(k, v)| {
            let expiry = request.expiry.get(&k).copied();
            (k, v.into_entry(expiry))
        })
        .collect();

    let cookie_count = cookies.len();
    let domain = request.domain.clone();

    // Create and store profile
    let profile = CookieProfile {
        domain: request.domain,
        cookies,
        created_at: Some({
            let dur = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            format!("{}", dur.as_secs())
        }),
        notes: Some("Imported via extension bridge".to_string()),
    };

    match store::store_profile(&profile) {
        Ok(()) => {
            info!(
                domain = %domain,
                cookies = cookie_count,
                "Stored profile via bridge"
            );
            (
                StatusCode::OK,
                Json(CookiesResponse {
                    ok: true,
                    domain,
                    cookies_stored: cookie_count,
                    error: None,
                }),
            )
        }
        Err(e) => {
            error!(domain = %domain, error = %e, "Failed to store profile");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CookiesResponse {
                    ok: false,
                    domain,
                    cookies_stored: 0,
                    error: Some(format!("Failed to store: {}", e)),
                }),
            )
        }
    }
}
