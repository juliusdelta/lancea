//! D‑Bus surface & orchestration glue (stubs for M0).

use std::cmp::Ordering;
use std::sync::atomic::AtomicU64;

use anyhow::Result;
use lancea_model::{Envelope, Outcome, Preview, ResolvedCommand, ResultItem, ResultsBatch};
use lancea_provider_emoji::EmojiProvider;
use lancea_registry::CommandRegistry;
use serde_json::json;
use tracing::{info, instrument};
use zbus::{interface, connection};
use zbus::object_server::{SignalEmitter};

pub struct EngineBus {
    registry: CommandRegistry,
    emoji: EmojiProvider,
    epoch: AtomicU64,
}

impl EngineBus {
    pub fn new() -> Self {
        Self {
            registry: CommandRegistry::new(),
            emoji: EmojiProvider::new().expect("Failed to initialize EmojiProvider"),
            epoch: AtomicU64::new(0),
        }
    }

    fn next_epoch(&self) -> u64 {
        self.epoch.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }

    fn parse_text_from_envelope(s: &str) -> String {
        serde_json::from_str::<Envelope<serde_json::Value>>(s)
            .ok()
            .and_then(|e| e.data.get("text").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .unwrap_or_default()
    }
}

#[interface(name = "org.lancea.Engine1")]
impl EngineBus {
    fn resolve_command(&self, text_json: &str) -> String {
        let text = Self::parse_text_from_envelope(text_json);
        let resolved: ResolvedCommand = self.registry.resolve(&text);

        return serde_json::to_string(&Envelope::wrap(resolved)).unwrap();
    }

    /// Search(args_json) -> token
    ///
    /// args_json envelope data:
    /// { "text": "/emoji laugh", "providerIds": ["emoji"], "epoch": <optional u64> }
    async fn search(
        &self,
        args_json: &str,
        #[zbus(signal_emitter)]
        emitter: SignalEmitter<'_>,
    ) -> u64 {
        let args: Envelope<serde_json::Value> = serde_json::from_str(args_json).unwrap_or_else(|_| Envelope { v: "1.0".into(), data: json!({}) });
        let text = args.data.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let epoch = args.data.get("epoch").and_then(|v| v.as_u64()).unwrap_or_else(|| self.next_epoch());
        let token = 1u64;

        let items: Vec<ResultItem> = self.emoji.search(&text);

        let reset_batch = Envelope::wrap(ResultsBatch::Reset { items });
        let reset_json = serde_json::to_string(&reset_batch).unwrap();
        let _ = Self::results_updated(
            &emitter,
            epoch,
            "emoji",
            token,
            &reset_json,
        );

        let end_batch = Envelope::wrap(ResultsBatch::End);
        let end_json = serde_json::to_string(&end_batch).unwrap();
        let _ = Self::results_updated(
            &emitter,
            epoch,
            "emoji",
            token,
            &end_json,
        );

        token
    }
    fn cancel(&self, _cancel_json: &str) { /* no-op for stub */
    }

    /// RequestPreview(args_json)
    ///
    /// args_json envelope data:
    /// { "providerId":"emoji", "key":"emoji:joy", "epoch": <u64> }
    async fn request_preview(
        &self,
        args_json: &str,
        #[zbus(signal_emitter)]
        emitter: SignalEmitter<'_>,
    ) { /* no-op for stub */
        let args: Envelope<serde_json::Value> = serde_json::from_str(args_json).unwrap_or_else(|_| Envelope { v: "1.0".into(), data: json!({}) });
        let epoch = args.data.get("epoch").and_then(|v| v.as_u64()).unwrap_or(self.epoch.load(std::sync::atomic::Ordering::SeqCst));
        let key = args.data.get("key").and_then(|v| v.as_str()).unwrap_or("");
        if key.is_empty() {
            return;
        }

        if let Some(preview) = self.emoji.preview(key) {
            let preview_json = serde_json::to_string(&Envelope::wrap(preview)).unwrap();
            let _ = Self::preview_updated(&emitter, epoch, "emoji", key, &preview_json).await;
        }
    }

    /// Execute(args_json) -> envelope(outcome)
    ///
    /// args_json envelope data:
    /// { "providerId":"emoji", "actionId":"copy_glyph", "key":"emoji:joy" }
    fn execute(&self, args_json: &str) -> String {
        let args: Envelope<serde_json::Value> = serde_json::from_str(args_json).unwrap_or_else(|_| Envelope { v: "1.0".into(), data: json!({}) });
        let action = args.data.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let key = args.data.get("key").and_then(|v| v.as_str()).unwrap_or("");

        let ok = match action {
            "copy_glyph" => self.emoji.execute_copy_glyph(key),
            "copy_shortcode" => self.emoji.execute_copy_glyph(key),
            _ => false,
        };

        let outcome = if ok {
            Outcome { status: "ok".into(), message: Some("Copied glyph requested".into()) }
        } else {
            Outcome { status: "error".into(), message: Some("Unknown action or key".into()) }
        };

        return serde_json::to_string(&Envelope::wrap(outcome)).unwrap()
    }

    #[zbus(signal)]
    async fn results_updated(
        #[zbus(signal_emitter)]
        emitter: &SignalEmitter<'_>,
        epoch: u64,
        provider_id: &str,
        token: u64,
        batch_json: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn preview_updated(
        #[zbus(signal_emitter)]
        emitter: &SignalEmitter<'_>,
        epoch: u64,
        provider_id: &str,
        result_key: &str,
        preview_json: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn preview_error(
        #[zbus(signal_emitter)]
        emitter: &SignalEmitter<'_>,
        epoch: u64,
        provider_id: &str,
        err_json: u64,
    ) -> zbus::Result<()>;
}

#[instrument(skip_all)]
pub async fn run_bus() -> Result<()> {
    let engine = EngineBus::new();

    let _conn = connection::Builder::session()?
        .name("org.lancea.Engine1")?
        .serve_at("/org/lancea/Engine1", engine)?
        .build()
        .await?;

    info!("Lancea engined is up on org.lancea.Engine1 at /org/lancea/Engine1");

    std::future::pending::<()>().await;
    return Ok(());
}
