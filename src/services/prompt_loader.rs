use std::{fs, path::Path, sync::Arc};

use rand::{Rng, seq::SliceRandom};

use crate::error::ClewdrError;

/// Loads banned prompt snippets from disk and generates randomized payloads.
#[derive(Clone)]
pub struct PromptLoader {
    prompts: Arc<Vec<String>>,
}

impl PromptLoader {
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, ClewdrError> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Err(ClewdrError::PathNotFound {
                msg: format!("Prompt directory not found: {}", dir.display()),
            });
        }
        let mut items = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file()
                && entry.path().extension().and_then(|e| e.to_str()) == Some("txt")
            {
                let content = fs::read_to_string(entry.path())?;
                if !content.trim().is_empty() {
                    items.push(content);
                }
            }
        }
        if items.is_empty() {
            return Err(ClewdrError::PathNotFound {
                msg: format!("No .txt prompt files found in directory: {}", dir.display()),
            });
        }
        Ok(Self {
            prompts: Arc::new(items),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }

    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    pub fn random_prompt(&self) -> Option<String> {
        if self.prompts.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let prompts = &self.prompts;
        let combined = if prompts.len() < 3 {
            prompts.choose(&mut rng)?.clone()
        } else {
            let pick = match rng.gen_range(0..100) {
                0..=19 => 1,
                20..=49 => 2,
                _ => 3,
            };
            let mut idx: Vec<_> = (0..prompts.len()).collect();
            idx.shuffle(&mut rng);
            idx.into_iter()
                .take(pick)
                .map(|i| prompts[i].clone())
                .collect::<Vec<_>>()
                .join("\n")
        };
        Some(format!("{combined}\n{}", random_suffix(30)))
    }
}

fn random_suffix(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
