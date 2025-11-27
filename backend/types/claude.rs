use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Message role in conversation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// Message content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text { content: String },
    Blocks { content: Vec<ContentBlock> },
}

/// Content block within a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Image { source: ImageSource },
}

/// Image source for content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub type_: String,
    pub media_type: String,
    pub data: String,
}

/// Message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    #[serde(flatten)]
    pub content: MessageContent,
}

impl Message {
    /// Create a new text message
    pub fn new_text(role: Role, text: impl Into<String>) -> Self {
        Self {
            role,
            content: MessageContent::Text {
                content: text.into(),
            },
        }
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,
}

/// Required parameters for creating a message
#[derive(Debug)]
pub struct RequiredMessageParams {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
}

/// Parameters for creating a message - simplified for ban operations
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateMessageParams {
    /// Maximum number of tokens to generate
    pub max_tokens: u32,
    /// Input messages for the conversation
    pub messages: Vec<Message>,
    /// Model to use
    pub model: String,
    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Value>,
    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl CreateMessageParams {
    /// Create a new message with required parameters
    pub fn new(required: RequiredMessageParams) -> Self {
        Self {
            max_tokens: required.max_tokens,
            messages: required.messages,
            model: required.model,
            system: None,
            stream: None,
        }
    }

    /// Set system prompt
    pub fn with_system(mut self, system: Value) -> Self {
        self.system = Some(system);
        self
    }

    /// Set streaming mode
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
}

impl Default for CreateMessageParams {
    fn default() -> Self {
        Self {
            max_tokens: 512, // Reduced default for ban operations
            messages: Vec::new(),
            model: "claude-sonnet-4-20250514".to_string(),
            system: None,
            stream: Some(false),
        }
    }
}

impl From<&str> for Message {
    fn from(text: &str) -> Self {
        Message::new_text(Role::User, text)
    }
}
