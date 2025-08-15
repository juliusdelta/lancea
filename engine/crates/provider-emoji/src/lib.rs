//! Emoji provider (stub for M0; real data later).

use anyhow::Result;
use lancea_model::{Preview, ResultItem};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EmojiRec {
    key: String,
    glyph: String,
    name: String,
    shortcodes: Vec<String>,
    keywords: Vec<String>,
}

pub struct EmojiProvider {
    // later: Vec<EmojiRec>,
}

impl EmojiProvider {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn search(&self, _query: &str) -> Vec<ResultItem> {
        vec![ResultItem {
            key: "emoji:joy".into(),
            title: "Face with Tears of Joy".into(),
            providerId: "emoji".into(),
            score: 1.0,
            extras: Some(serde_json::json!({
                "glyph": "😂",
                "shortcodes": [":joy:", ":face_with_tears_of_joy:"],
                "keywords": ["happy", "funny", "laugh"]
            })),
        }]
    }

    pub fn preview(&self, _key: &str) -> Preview {
        Preview {
            previewKind: "card".into(),
            data: serde_json::json!({
                "glyph": "😂",
                "title": "Face with Tears of Joy",
            }),
        }
    }

    pub fn execute_copy_glyph(&self, _key: &str) -> bool {
        true
    }
}
