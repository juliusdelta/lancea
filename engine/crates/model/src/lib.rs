//! Core data model for Lancea M0.

use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1.0";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Envelope<T> {
    pub v: String,
    pub data: T,
}

impl<T> Envelope<T> {
    pub fn wrap(data: T) -> Self {
        Self {
            v: API_VERSION.to_string(),
            data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResolvedCommand {
    pub matched: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providerId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commandId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResultItem {
    pub key: String,
    pub title: String,
    pub providerId: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum ResultsBatch {
    #[serde(rename = "reset")]
    Reset { items: Vec<ResultItem> },
    #[serde(rename = "insert")]
    Insert { at: usize, items: Vec<ResultItem> },
    #[serde(rename = "end")]
    End,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preview {
    pub previewKind: String, // "card" for M0
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Outcome {
    pub status: String, // "ok" | "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
