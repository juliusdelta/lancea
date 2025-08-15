//! D‑Bus surface & orchestration glue (stubs for M0).

use anyhow::Result;
use lancea_model::{Envelope, Outcome, Preview, ResolvedCommand};
use lancea_registry::CommandRegistry;
use serde_json::json;
use tracing::{info, instrument};
use zbus::{dbus_interface, ConnectionBuilder, ObjectServer};

pub struct EngineBus {
    registry: CommandRegistry,
}

impl EngineBus {
    pub fn new() -> Self { Self { registry: CommandRegistry::new() } }
}

#[dbus_interface(name = "org.lancea.Engine1")]
impl EngineBus {
    fn resolve_command(&self, text_json: &str) -> String {
        let text: String = serde_json::from_str::<Envelope<serde_json::Value>>(text_json)
            .ok()
            .and_then(|e| e.data.get("text").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .unwrap_or_default();

        let resolved: ResolvedCommand = self.registry.resolve(&text);

        return serde_json::to_string(&Envelope::wrap(resolved)).unwrap()
    }

    fn search(&self, _args_json: &str) -> u64 { 1 }
    fn cancel(&self, _cancel_json: &str) { /* no-op for stub */ }
    fn request_preview(&self, _args_json: &str) { /* no-op for stub */ }
    fn execute(&self, _args_json: &str) -> String {
        serde_json::to_string(&Envelope::wrap(Outcome {
            status: "ok".into(),
            message: Some("stubbed".into()),
        })).unwrap()
    }
    #[dbus_interface(signal)]
    async fn results_updated(ctxt: &zbus::SignalContext<'_>, _epoch: u64, _provider_id: &str, _token: u64, _batch_json: &str) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn preview_updated(ctxt: &zbus::SignalContext<'_>, _epoch: u64, _provider_id: &str, _result_key: u64, _preview_json: &str) -> zbus::Result<()>;

    #[dbus_interface(signal)]
    async fn preview_error(ctxt: &zbus::SignalContext<'_>, _epoch: u64, _provider_id: &str, _err_json: u64) -> zbus::Result<()>;
}

#[instrument(skip_all)]
pub async fn run_bus() -> Result<()> {
    let engine = EngineBus::new();

    let _conn = ConnectionBuilder::session()?
        .name("org.lancea.Engine1")?
        .serve_at("/org/lancea/Engine1", engine)?
        .build()
        .await?;

    info!("Lancea engined is up on org.lancea.Engine1 at /org/lancea/Engine1");

    std::future::pending::<()>().await;
    return Ok(())
}
