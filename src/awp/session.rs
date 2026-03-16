use std::sync::Arc;

use reqwest::cookie::Jar;
use reqwest::Client;

use crate::network::fetch;
use crate::som::types::Som;

/// An in-memory browsing session.
pub struct Session {
    pub id: String,
    pub user_agent: String,
    pub locale: String,
    pub timeout_ms: u64,
    pub cookie_jar: Arc<Jar>,
    pub client: Client,
    pub current_som: Option<Som>,
    pub current_html: Option<String>,
    pub current_url: Option<String>,
}

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

impl Session {
    pub fn new(
        id: String,
        user_agent: Option<String>,
        locale: Option<String>,
        timeout_ms: Option<u64>,
    ) -> Result<Self, String> {
        let ua = user_agent.unwrap_or_else(|| DEFAULT_USER_AGENT.to_string());
        let locale = locale.unwrap_or_else(|| "en-US".to_string());
        let timeout = timeout_ms.unwrap_or(30000);
        let jar = Arc::new(Jar::default());
        let client = fetch::build_client(Some(&ua), jar.clone())
            .map_err(|e| e.to_string())?;

        Ok(Session {
            id,
            user_agent: ua,
            locale,
            timeout_ms: timeout,
            cookie_jar: jar,
            client,
            current_som: None,
            current_html: None,
            current_url: None,
        })
    }
}
