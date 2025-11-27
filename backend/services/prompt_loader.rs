use std::{fs, path::Path, sync::Arc};

use rand::{seq::SliceRandom, Rng};

use crate::error::ClewdrError;

/// Loads banned prompt snippets from disk and generates randomized payloads.
/// Simplified to use single prompt with random suffix for better performance.
#[derive(Clone)]
pub struct PromptLoader {
    prompts: Arc<Vec<String>>,
}

impl PromptLoader {
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, ClewdrError> {
        let dir = dir.as_ref();

        // 若目录不存在则尝试创建，避免服务启动失败
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        let mut items = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file()
                && entry.path().extension().and_then(|e| e.to_str()) == Some("txt")
            {
                let content = fs::read_to_string(entry.path())?;
                if !content.trim().is_empty() {
                    items.push(content.trim().to_string());
                }
            }
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

    /// Get a random prompt with a random suffix
    pub fn random_prompt(&self) -> Option<String> {
        if self.prompts.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let prompt = self.prompts.choose(&mut rng)?.clone();
        let suffix = self.random_suffix(10); // Reduced suffix length

        Some(format!("{}\n\n{}", prompt, suffix))
    }

    // Private helper to generate random suffix
    fn random_suffix(&self, len: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..len)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}
