//! D‑Bus surface & orchestration glue (stubs for M0).

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

use anyhow::Result;
use lancea_model::{Envelope, Outcome, Provider, ResolvedCommand, ResultItem, ResultsBatch};
use lancea_provider_apps::AppsProvider;
use lancea_provider_emoji::EmojiProvider;
use lancea_registry::CommandRegistry;
use serde_json::json;
use tracing::{info, instrument};
use zbus::object_server::SignalEmitter;
use zbus::{connection, interface};

pub struct EngineBus {
    registry: CommandRegistry,
    providers: HashMap<String, Box<dyn Provider>>,
    epoch: AtomicU64,
}

impl EngineBus {
    pub fn new() -> Self {
        let mut providers: HashMap<String, Box<dyn Provider>> = HashMap::new();

        let emoji = EmojiProvider::new().expect("Failed to initialize EmojiProvider");
        let apps = AppsProvider::new().expect("Apps scan");

        providers.insert(emoji.id().to_string(), Box::new(emoji));
        providers.insert(apps.id().to_string(), Box::new(apps));

        Self {
            registry: CommandRegistry::new(),
            providers,
            epoch: AtomicU64::new(0),
        }
    }

    fn next_epoch(&self) -> u64 {
        self.epoch.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }

    fn parse_text_from_envelope(s: &str) -> String {
        serde_json::from_str::<Envelope<serde_json::Value>>(s)
            .ok()
            .and_then(|e| {
                e.data
                    .get("text")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
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
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> u64 {
        dbg!(
            "[EngineBus#search] - Called search with args: {}",
            args_json
        );
        let args: Envelope<serde_json::Value> =
            serde_json::from_str(args_json).unwrap_or_else(|_| Envelope {
                v: "1.0".into(),
                data: json!({}),
            });
        let text = args
            .data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let epoch = args
            .data
            .get("epoch")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| self.next_epoch());

        let provider_ids: Vec<String> = args
            .data
            .get("providerIds")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .filter(|vec: &Vec<String>| !vec.is_empty())
            .unwrap_or_else(|| vec!["apps".to_string()]);

        dbg!(
            "############[EngineBus#search] - providerIds: {:?}",
            &provider_ids
        );
        let token = 1u64;

        // Use the first provider ID for searching
        let provider_id = &provider_ids[0];
        let items: Vec<ResultItem> = if let Some(provider) = self.providers.get(provider_id) {
            dbg!(
                "[EngineBus#search] - Provider '{}' resolved from command.",
                provider_id
            );
            provider.search(&text)
        } else {
            dbg!(
                "[EngineBus#search] - Unknown provider '{}', falling back to apps",
                provider_id
            );
            self.providers
                .get("apps")
                .map(|p| p.search(&text))
                .unwrap_or_default()
        };

        dbg!(
            "[EngineBus#search] - Found {} items for query '{}'",
            &items.len(),
            text
        );
        let reset_batch = Envelope::wrap(ResultsBatch::Reset { items });
        let reset_json = serde_json::to_string(&reset_batch).unwrap();

        dbg!(
            "[EngineBus#search] - Emitting ResultsUpdated / reset_batch for epoch {}",
            &epoch
        );
        let _ = Self::results_updated(&emitter, epoch, provider_id, token, &reset_json).await;

        let end_batch = Envelope::wrap(ResultsBatch::End);
        let end_json = serde_json::to_string(&end_batch).unwrap();

        dbg!(
            "[EngineBus#search] - Emitting ResultsUpdated / end_batch {}",
            &epoch
        );
        let _ = Self::results_updated(&emitter, epoch, provider_id, token, &end_json).await;

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
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) {
        dbg!(
            "[EngineBus#request_preview] - Called request_preview with args: {}",
            &args_json
        );
        let args: Envelope<serde_json::Value> =
            serde_json::from_str(args_json).unwrap_or_else(|_| Envelope {
                v: "1.0".into(),
                data: json!({}),
            });
        let epoch = args
            .data
            .get("epoch")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.epoch.load(std::sync::atomic::Ordering::SeqCst));
        let key = args.data.get("key").and_then(|v| v.as_str()).unwrap_or("");
        if key.is_empty() {
            return;
        }

        // Determine provider from key prefix (e.g., "emoji:joy" -> "emoji")
        let provider_id = key.split(':').next().unwrap_or("");

        if let Some(provider) = self.providers.get(provider_id) {
            if let Some(preview) = provider.preview(key) {
                let preview_json = serde_json::to_string(&Envelope::wrap(preview)).unwrap();

                dbg!(
                    "[EngineBus#request_preview] - Emitting PreviewUpdated for provider '{}' at epoch: {}",
                    provider_id,
                    &epoch
                );
                let _ =
                    Self::preview_updated(&emitter, epoch, provider_id, key, &preview_json).await;
            }
        } else {
            dbg!(
                "[EngineBus#request_preview] - Unknown provider '{}' for key '{}'",
                provider_id,
                key
            );
        }
    }

    /// Execute(args_json) -> envelope(outcome)
    ///
    /// args_json envelope data:
    /// { "providerId":"emoji", "actionId":"copy_glyph", "key":"emoji:joy" }
    fn execute(&self, args_json: &str) -> String {
        dbg!(
            "[EngineBus#execute] - Called execute with args: {}",
            args_json
        );
        let args: Envelope<serde_json::Value> =
            serde_json::from_str(args_json).unwrap_or_else(|_| Envelope {
                v: "1.0".into(),
                data: json!({}),
            });
        let action = args
            .data
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let key = args.data.get("key").and_then(|v| v.as_str()).unwrap_or("");

        // Determine provider from key prefix (e.g., "emoji:joy" -> "emoji")
        let provider_id = key.split(':').next().unwrap_or("");

        let ok = if let Some(provider) = self.providers.get(provider_id) {
            dbg!(
                "[EngineBus#execute] - Executing action '{}' on key '{}' with provider '{}'",
                action,
                key,
                provider_id
            );
            provider.execute(action, key)
        } else {
            dbg!(
                "[EngineBus#execute] - Unknown provider '{}' for key '{}'",
                provider_id,
                key
            );
            false
        };

        let outcome = if ok {
            Outcome {
                status: "ok".into(),
                message: Some(format!("Action '{}' executed successfully", action)),
            }
        } else {
            Outcome {
                status: "error".into(),
                message: Some(format!(
                    "Failed to execute action '{}' or unknown provider/key",
                    action
                )),
            }
        };

        dbg!("[EngineBus#execute] - Outcome: {:?}", &outcome);
        return serde_json::to_string(&Envelope::wrap(outcome)).unwrap();
    }

    #[zbus(signal)]
    async fn results_updated(
        #[zbus(signal_emitter)] emitter: &SignalEmitter<'_>,
        epoch: u64,
        provider_id: &str,
        token: u64,
        batch_json: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn preview_updated(
        #[zbus(signal_emitter)] emitter: &SignalEmitter<'_>,
        epoch: u64,
        provider_id: &str,
        result_key: &str,
        preview_json: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn preview_error(
        #[zbus(signal_emitter)] emitter: &SignalEmitter<'_>,
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
