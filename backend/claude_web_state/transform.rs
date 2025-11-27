use std::{collections::HashMap, fmt::Write, mem};

use serde_json::Value;

use crate::{
    claude_web_state::ClaudeWebState,
    types::{
        claude::{ContentBlock, CreateMessageParams, Message, MessageContent, Role},
        claude_web::request::*,
    },
};

impl ClaudeWebState {
    pub fn transform_request(&self, mut value: CreateMessageParams) -> Option<WebRequestBody> {
        let system = value.system.take();
        let msgs = mem::take(&mut value.messages);
        let system = merge_system(system.unwrap_or_default());
        let merged = merge_messages(msgs, system)?;

        let tools = vec![];
        Some(WebRequestBody {
            max_tokens_to_sample: value.max_tokens,
            attachments: vec![Attachment::new(merged.paste)],
            files: vec![],
            model: Some(value.model), // Always include model for ban operations
            rendering_mode: if value.stream.unwrap_or_default() {
                "messages".to_string()
            } else {
                "raw".to_string()
            },
            prompt: merged.prompt,
            timezone: "America/New_York".to_string(),
            images: vec![], // No images for ban operations
            tools,
        })
    }
}

/// Merged messages
#[derive(Default, Debug)]
struct Merged {
    pub paste: String,
    pub prompt: String,
}

/// Merges multiple messages into a single text prompt, handling system instructions
///
/// # Arguments
/// * `msgs` - Vector of messages to merge
/// * `system` - System instructions to prepend
///
/// # Returns
/// * `Option<Merged>` - Merged prompt text, or None if merging fails
fn merge_messages(msgs: Vec<Message>, system: String) -> Option<Merged> {
    if msgs.is_empty() {
        return None;
    }
    let h = "Human".to_string();
    let a = "Assistant".to_string();
    let line_breaks = "\n\n";
    let system = system.trim().to_string();
    // Estimate capacity: average 100 chars per message
    let size = msgs.len() * 100;
    // preallocate string to avoid reallocations
    let mut w = String::with_capacity(size);

    let chunks = msgs
        .into_iter()
        .filter_map(|m| match m.content {
            MessageContent::Blocks { content } => {
                // collect all text blocks, join them with new line
                let blocks = content
                    .into_iter()
                    .filter_map(|b| match b {
                        ContentBlock::Text { text } => Some(text.trim().to_string()),
                        _ => None, // Ignore all non-text blocks for ban operations
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if blocks.is_empty() {
                    None
                } else {
                    Some((m.role, blocks))
                }
            }
            MessageContent::Text { content } => {
                // plain text
                let content = content.trim().to_string();
                if content.is_empty() {
                    None
                } else {
                    Some((m.role, content))
                }
            }
        })
        // group by role and collect
        .collect::<Vec<_>>();

    // Group messages by role
    let mut grouped: HashMap<Role, Vec<String>> = HashMap::new();
    for (role, text) in chunks {
        grouped.entry(role).or_default().push(text);
    }

    // Create messages from grouped content
    let mut msgs = grouped.into_iter().map(|(role, texts)| {
        let txt = texts.join("\n");
        (role, txt)
    });
    // first message does not need prefix
    if !system.is_empty() {
        w += system.as_str();
    } else {
        let first = msgs.next()?;
        w += first.1.as_str();
    }
    for (role, text) in msgs {
        let prefix = match role {
            Role::System => {
                tracing::warn!("System message should be merged into the first message");
                continue;
            }
            Role::User => format!("{h}: "),
            Role::Assistant => format!("{a}: "),
        };
        write!(w, "{line_breaks}{prefix}{text}").ok()?;
    }

    // prompt polyfill
    let p = String::new();

    Some(Merged {
        paste: w,
        prompt: p,
    })
}

/// Merges system message content into a single string
/// Handles both string and array formats for system messages
///
/// # Arguments
/// * `sys` - System message content as a JSON Value
///
/// # Returns
/// Merged system message as a string
fn merge_system(sys: Value) -> String {
    match sys {
        Value::String(s) => s,
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v["text"].as_str())
            .map(|v| v.trim())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}
