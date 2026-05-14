//! MCP JSON-RPC protocol-level tests.
//!
//! These tests exercise the dispatch surface in `mcp::handle_line` without
//! touching the network. Tool calls that would normally reach Google fail
//! because no credentials are available, which is fine for parameter
//! validation tests — we only check that the framing is correct.

#![allow(dead_code)]

use serde_json::{json, Value};
use std::sync::Arc;

// Pull in the binary crate's modules as a library for testing.
#[path = "../src/auth.rs"]
mod auth;
#[path = "../src/gcal.rs"]
mod gcal;
#[path = "../src/mcp.rs"]
mod mcp;
#[path = "../src/tools.rs"]
mod tools;

fn fresh_state() -> Arc<mcp::ServerState> {
    // Force the token dir to a tempdir so tests do not touch the real home.
    let tmp = std::env::temp_dir().join(format!("gcal-mcp-test-{}", uuid_like()));
    std::fs::create_dir_all(&tmp).unwrap();
    Arc::new(mcp::ServerState {
        http: reqwest::Client::new(),
        token_dir: tmp,
        credentials_path: None,
    })
}

fn uuid_like() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos}-{}", std::process::id())
}

async fn dispatch(state: Arc<mcp::ServerState>, request: Value) -> Value {
    let line = serde_json::to_string(&request).unwrap();
    let raw = mcp::handle_line(state, &line).await.unwrap_or_default();
    serde_json::from_str(&raw).unwrap()
}

#[tokio::test]
async fn test_initialize() {
    let state = fresh_state();
    let resp = dispatch(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }),
    )
    .await;

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    let result = &resp["result"];
    assert!(!result.is_null(), "initialize must return a result");
    assert_eq!(result["serverInfo"]["name"], "gcal-mcp");
    assert!(result["capabilities"]["tools"].is_object());
    assert!(result["protocolVersion"].is_string());
}

#[tokio::test]
async fn test_tools_list() {
    let state = fresh_state();
    let resp = dispatch(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": "list",
            "method": "tools/list"
        }),
    )
    .await;

    let tools = resp["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 11, "expected 11 tools");

    let names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap_or(""))
        .collect();
    for expected in [
        "list-calendars",
        "list-events",
        "search-events",
        "get-event",
        "create-event",
        "update-event",
        "delete-event",
        "respond-to-event",
        "get-freebusy",
        "get-current-time",
        "manage-accounts",
    ] {
        assert!(
            names.contains(&expected),
            "missing tool {expected}; got {names:?}"
        );
    }

    for t in tools {
        let desc = t["description"].as_str().unwrap_or("");
        assert!(!desc.is_empty(), "tool {} has empty description", t["name"]);
        assert!(
            t["inputSchema"].is_object(),
            "tool {} missing inputSchema",
            t["name"]
        );
    }
}

#[tokio::test]
async fn test_unknown_method() {
    let state = fresh_state();
    let resp = dispatch(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "this/does/not/exist"
        }),
    )
    .await;

    let err = &resp["error"];
    assert_eq!(err["code"], -32601);
    assert!(err["message"]
        .as_str()
        .unwrap_or("")
        .contains("Method not found"));
}

#[tokio::test]
async fn test_invalid_json() {
    let state = fresh_state();
    let raw = mcp::handle_line(state, "{not really json").await.unwrap();
    let resp: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(resp["error"]["code"], -32700);
}

#[tokio::test]
async fn test_tools_call_missing_params() {
    let state = fresh_state();
    let resp = dispatch(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "list-events",
                "arguments": {}
            }
        }),
    )
    .await;

    let err = &resp["error"];
    assert_eq!(err["code"], -32602);
    assert!(err["message"]
        .as_str()
        .unwrap_or("")
        .contains("calendar_id"));
}

#[tokio::test]
async fn test_initialized_is_notification() {
    let state = fresh_state();
    let line = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }))
    .unwrap();
    let result = mcp::handle_line(state, &line).await;
    assert!(
        result.is_none(),
        "notifications must not produce a response"
    );
}

#[tokio::test]
async fn test_get_current_time_via_dispatch() {
    let state = fresh_state();
    let resp = dispatch(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": "now",
            "method": "tools/call",
            "params": {
                "name": "get-current-time",
                "arguments": {}
            }
        }),
    )
    .await;

    let content = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("text content");
    assert!(content.contains("iso8601"));
}
