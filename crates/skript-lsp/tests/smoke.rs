//! Smoke test: connect to a running skript-lsp server over WebSocket,
//! send `initialize`, and verify a well-formed JSON-RPC response comes back.
//!
//! Run the server first: `cargo run -p skript-lsp -- --port 9876`
//! Then: `cargo test -p skript-lsp --test smoke -- --ignored --nocapture`

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

const URL: &str = "ws://127.0.0.1:9876";

#[tokio::test]
#[ignore]
async fn ws_initialize_handshake_round_trip() {
    let (mut ws, _resp) = tokio_tungstenite::connect_async(URL)
        .await
        .expect("websocket connect failed — is the server running on 9876?");

    let init = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": null,
            "capabilities": {}
        }
    });
    ws.send(Message::Text(init.to_string()))
        .await
        .expect("send initialize");

    // Read the response (control frames may interleave).
    let mut got = None;
    for _ in 0..10 {
        match ws.next().await {
            Some(Ok(Message::Text(t))) => {
                got = Some(t);
                break;
            }
            Some(Ok(Message::Binary(b))) => {
                got = Some(String::from_utf8(b.to_vec()).unwrap());
                break;
            }
            Some(Ok(_)) => continue,
            Some(Err(e)) => panic!("ws read error: {e}"),
            None => panic!("ws closed before response"),
        }
    }
    let text = got.expect("no response text frame received");
    let resp: serde_json::Value = serde_json::from_str(&text).expect("response not JSON");
    assert_eq!(resp["jsonrpc"], "2.0", "bad jsonrpc field: {resp}");
    assert_eq!(resp["id"], 1, "bad id field: {resp}");
    assert!(
        resp.get("result").is_some(),
        "expected `result`, got error: {resp}"
    );
    let caps = &resp["result"]["capabilities"];
    assert_eq!(caps["hoverProvider"], true, "hover capability missing");
    assert!(
        caps["semanticTokensProvider"].is_object(),
        "semanticTokens capability missing"
    );
    let server_info = &resp["result"]["serverInfo"];
    assert_eq!(server_info["name"], "skript-lsp");
    println!("smoke test OK: server_info = {server_info}");
}
