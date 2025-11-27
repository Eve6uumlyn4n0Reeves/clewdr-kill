use axum::http::HeaderValue;
use snafu::ResultExt;
use wreq::{ClientBuilder, Method, Response};
use wreq_util::Emulation;

use super::ClaudeWebState;
use crate::{
    config::CLEWDR_CONFIG,
    error::{CheckClaudeErr, ClewdrError, WreqSnafu},
    types::claude::CreateMessageParams,
};

impl ClaudeWebState {
    /// Create a minimal state for ban operations - no org/conv tracking needed
    pub fn new_minimal() -> Self {
        ClaudeWebState {
            cookie: None,
            org_uuid: None,
            conv_uuid: None,
            cookie_header_value: HeaderValue::from_static(""),
            capabilities: Vec::new(),
            endpoint: CLEWDR_CONFIG.load().endpoint(),
            proxy: CLEWDR_CONFIG.load().wreq_proxy.to_owned(),
            api_format: super::ClaudeApiFormat::Claude,
            stream: false,
            client: super::SUPER_CLIENT.to_owned(),
            key: None,
            usage: crate::types::claude::Usage::default(),
            last_params: None,
            user_agent: None,
        }
    }

    /// Set cookie without validation - for ban operations
    pub fn set_cookie(&mut self, cookie_str: &str) -> Result<(), ClewdrError> {
        self.cookie_header_value = HeaderValue::from_str(cookie_str)?;
        self.rebuild_client()?;
        Ok(())
    }

    pub fn set_proxy(&mut self, proxy: wreq::Proxy) -> Result<(), ClewdrError> {
        self.proxy = Some(proxy);
        self.rebuild_client()
    }

    pub fn set_user_agent(&mut self, ua: String) {
        self.user_agent = Some(ua);
    }

    fn rebuild_client(&mut self) -> Result<(), ClewdrError> {
        // Rebuild client with current proxy & UA
        let mut client_builder = ClientBuilder::new()
            .cookie_store(true)
            .emulation(Emulation::Chrome136);

        if let Some(ref proxy) = self.proxy {
            client_builder = client_builder.proxy(proxy.to_owned());
        }

        self.client = client_builder.build().context(WreqSnafu {
            msg: "Failed to build client",
        })?;

        if !self.cookie_header_value.as_bytes().is_empty() {
            self.client
                .set_cookie(&self.endpoint, &self.cookie_header_value);
        }

        Ok(())
    }

    /// Send raw request without bootstrap checks or chat management
    pub async fn send_raw(&mut self, params: CreateMessageParams) -> Result<Response, ClewdrError> {
        // Create a throwaway conversation UUID
        let conv_uuid = uuid::Uuid::new_v4().to_string();
        self.conv_uuid = Some(conv_uuid.clone());

        // Get an org UUID if we don't have one - try to extract from bootstrap
        if self.org_uuid.is_none() {
            if let Ok(org_uuid) = self.fetch_org_uuid().await {
                self.org_uuid = Some(org_uuid);
            } else {
                // If we can't get org, just fail fast
                return Err(ClewdrError::BadRequest {
                    msg: "Failed to get organization UUID".into(),
                });
            }
        }

        let org_uuid = self.org_uuid.as_ref().ok_or(ClewdrError::UnexpectedNone {
            msg: "Organization UUID not set".into(),
        })?;

        // Create conversation (no settings needed)
        let endpoint = self
            .endpoint
            .join(&format!(
                "api/organizations/{}/chat_conversations",
                org_uuid
            ))
            .map_err(|_| ClewdrError::BadRequest {
                msg: "Failed to construct conversation endpoint URL".into(),
            })?;

        let body = serde_json::json!({
            "uuid": conv_uuid,
            "name": "ban",
        });

        self.build_request(Method::POST, endpoint)
            .json(&body)
            .send()
            .await
            .context(WreqSnafu {
                msg: "Failed to create conversation",
            })?
            .check_claude()
            .await?;

        // Transform request (minimal)
        let mut body = self
            .transform_request(params)
            .ok_or(ClewdrError::BadRequest {
                msg: "Request body is empty".into(),
            })?;

        // No image uploads for ban operations
        body.images.clear();

        // Send completion request
        let endpoint = self
            .endpoint
            .join(&format!(
                "api/organizations/{}/chat_conversations/{}/completion",
                org_uuid, conv_uuid
            ))
            .map_err(|_| ClewdrError::BadRequest {
                msg: "Failed to construct completion endpoint URL".into(),
            })?;

        let response = self
            .build_request(Method::POST, endpoint)
            .json(&body)
            .send()
            .await
            .context(WreqSnafu {
                msg: "Failed to send completion",
            })?
            .check_claude()
            .await?;

        // No cleanup - just let conversations accumulate (they'll ban the account anyway)
        Ok(response)
    }

    /// Minimal org UUID fetch - no capability checks
    pub async fn fetch_org_uuid(&self) -> Result<String, ClewdrError> {
        let endpoint =
            self.endpoint
                .join("api/organizations")
                .map_err(|_| ClewdrError::BadRequest {
                    msg: "Failed to construct organizations endpoint URL".into(),
                })?;

        let res = self
            .build_request(Method::GET, endpoint)
            .send()
            .await
            .context(WreqSnafu {
                msg: "Failed to get organizations",
            })?
            .check_claude()
            .await?;

        let json = res.json::<serde_json::Value>().await.context(WreqSnafu {
            msg: "Failed to parse organizations",
        })?;

        let uuid = json
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|org| org.get("uuid"))
            .and_then(|u| u.as_str())
            .ok_or(ClewdrError::UnexpectedNone {
                msg: "No organization UUID found".into(),
            })?;

        Ok(uuid.to_string())
    }
}
