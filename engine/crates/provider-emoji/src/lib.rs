use anyhow::Result;
use lancea_model::{Preview, ResultItem, Provider};
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
    data: Vec<EmojiRec>,
}

impl EmojiProvider {
    pub fn new() -> Result<Self> {
        let raw = r#"
        [
          { "key":"emoji:joy",   "glyph":"😂", "name":"Face with Tears of Joy",
            "shortcodes":["joy","lol"], "keywords":["laugh","happy","tears"] },
          { "key":"emoji:smile", "glyph":"🙂", "name":"Slightly Smiling Face",
            "shortcodes":["slight_smile"], "keywords":["smile","happy"] },
          { "key":"emoji:grin",  "glyph":"😁", "name":"Beaming Face with Smiling Eyes",
            "shortcodes":["grin"], "keywords":["grin","smile","happy","teeth"] }
        ]
        "#;
        let data: Vec<EmojiRec> = serde_json::from_str(raw)?;
        Ok(Self { data })
    }

    pub fn search(&self, query: &str) -> Vec<ResultItem> {
        dbg!("EmojiProvider::search called with query: {}", query);
        let q = query.trim().to_lowercase();

        let q = q
            .strip_prefix("/emoji")
            .or_else(|| q.strip_prefix("/em"))
            .map(|s| s.trim())
            .unwrap_or(q.as_str());

        let mut items: Vec<(f32, ResultItem)> = Vec::new();
        for rec in &self.data {
            let mut score = None::<f32>;

            if q.is_empty() {
                score = Some(0.1);
            } else if rec.shortcodes.iter().any(|s| s == &q) {
                score = Some(1.0);
            } else if rec.name.to_lowercase().starts_with(q)
                || rec.keywords.iter().any(|k| k.starts_with(q))
            {
                score = Some(0.8);
            } else if rec.name.to_lowercase().contains(&q)
                || rec.keywords.iter().any(|k| k.contains(&q))
            {
                score = Some(0.4);
            }

            if let Some(s) = score {
                items.push((
                    s,
                    ResultItem {
                        key: rec.key.clone(),
                        title: rec.name.clone(),
                        provider_id: "emoji".into(),
                        score: s,
                        extras: Some(serde_json::json!({
                            "glyph": rec.glyph,
                            "shortcodes": rec.shortcodes.get(0),
                        })),
                    },
                ))
            }
        }

        items.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.1.title.cmp(&b.1.title))
        });

        return items.into_iter().map(|(_, ri)| ri).take(20).collect();
    }

    pub fn preview(&self, key: &str) -> Option<Preview> {
        return self.data.iter().find(|r| r.key == key).map(|rec| Preview {
            preview_kind: "card".into(),
            data: serde_json::json!({
                "glyph": rec.glyph,
                "title": rec.name,
                "shortcodes": rec.shortcodes.get(0),
                "keywords": rec.keywords,
            }),
        });
    }

    pub fn execute_copy_glyph(&self, key: &str) -> bool {
        return self.data.iter().any(|r| r.key == key);
    }
}

impl Provider for EmojiProvider {
    fn id(&self) -> &str {
        "emoji"
    }

    fn search(&self, query: &str) -> Vec<ResultItem> {
        self.search(query)
    }

    fn preview(&self, key: &str) -> Option<Preview> {
        self.preview(key)
    }

    fn execute(&self, action: &str, key: &str) -> bool {
        match action {
            "copy_glyph" => self.execute_copy_glyph(key),
            "copy_shortcode" => self.execute_copy_glyph(key),
            _ => false,
        }
    }
}
