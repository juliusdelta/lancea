use serde_json::json;
use tokio::time::{timeout, Duration};
use futures_lite::stream::StreamExt;
use zbus::{connection, proxy};

#[tokio::test]
async fn emoji_search_returns_results() {
    // Build a local (private) session bus and attach our server to it
    let server_conn = connection::Builder::session().unwrap().build().await.unwrap();

    // Serve engined at a unique name/path on the local connection
    let engine = lancea_bus::EngineBus::new();
    server_conn.object_server().at("/org/lancea/Engine1", engine).await.unwrap();
    server_conn.request_name("org.lancea.Engine1").await.unwrap();

    // Client on same local bus
    let proxy: proxy::Proxy = proxy::Builder::new(&server_conn)
        .destination("org.lancea.Engine1").unwrap()
        .path("/org/lancea/Engine1").unwrap()
        .interface("org.lancea.Engine1").unwrap()
        .build().await.unwrap();

    let env = json!({
        "v": "1.0",
        "data": {
            "text": "/emoji laugh"
        }
    });

    let resolved: String = proxy.call("ResolveCommand", &(env.to_string())).await.unwrap();

    let v: serde_json::Value = serde_json::from_str(&resolved).unwrap();
    assert!(v["data"]["matched"].as_bool().unwrap());

    let mut signals = proxy.receive_signal("ResultsUpdated").await.unwrap();

    let search_env = json!({
        "v": "1.0",
        "data": {
            "text": "/emoji laugh",
            "providerIds": ["emoji"],
        }
    });
    let _token: u64 = proxy.call("Search", &(search_env.to_string())).await.unwrap();

    let msg = timeout(Duration::from_secs(4), signals.next())
        .await
        .expect("signal timeout")
        .expect("stream ended unexpectedly");

    let (epoch, provider_id, token, batch_json): (u64, String, u64, String) = msg.body().deserialize().unwrap();

    assert_eq!(provider_id, "emoji");
    assert_eq!(token, 1);
    assert!(epoch >= 1);

    let batch: serde_json::Value = serde_json::from_str(&batch_json).unwrap();
    assert_eq!(batch["data"]["kind"], "reset");
    let items = batch["data"]["items"].as_array().unwrap();
    assert!(items.iter().any(|it| it["key"] == "emoji:joy"));
}
