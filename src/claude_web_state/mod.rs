use std::sync::LazyLock;

use axum::http::HeaderValue;
use url::Url;
use wreq::{
    Client, IntoUrl, Method, RequestBuilder,
    header::{ORIGIN, REFERER, USER_AGENT},
};

use crate::{
    config::CLAUDE_ENDPOINT,
    types::claude::CreateMessageParams,
};

pub mod client;
mod transform;

pub static SUPER_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

/// Minimal state for ban operations
#[derive(Clone)]
pub struct ClaudeWebState {
    pub cookie: Option<String>,
    cookie_header_value: HeaderValue,
    pub org_uuid: Option<String>,
    pub conv_uuid: Option<String>,
    pub capabilities: Vec<String>,
    pub endpoint: Url,
    pub proxy: Option<wreq::Proxy>,
    pub api_format: ClaudeApiFormat,
    pub stream: bool,
    pub client: Client,
    pub key: Option<(u64, usize)>,
    pub usage: crate::types::claude::Usage,
    pub last_params: Option<CreateMessageParams>,
    pub user_agent: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClaudeApiFormat {
    Claude,
}

impl ClaudeWebState {
    /// Build a request with the current cookie
    pub fn build_request(&self, method: Method, url: impl IntoUrl) -> RequestBuilder {
        self.client
            .set_cookie(&self.endpoint, &self.cookie_header_value);
        let req = self
            .client
            .request(method, url)
            .header(ORIGIN, CLAUDE_ENDPOINT);
        let req = if let Some(ref ua) = self.user_agent {
            req.header(USER_AGENT, ua)
        } else {
            req
        };
        if let Some(uuid) = self.conv_uuid.to_owned() {
            req.header(
                REFERER,
                self.endpoint
                    .join(&format!("chat/{uuid}"))
                    .map(|u| u.into())
                    .unwrap_or_else(|_| format!("{CLAUDE_ENDPOINT}chat/{uuid}")),
            )
        } else {
            req.header(
                REFERER,
                self.endpoint
                    .join("new")
                    .map(|u| u.into())
                    .unwrap_or_else(|_| format!("{CLAUDE_ENDPOINT}new")),
            )
        }
    }
}
