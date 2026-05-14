//! Validation and business-logic tests for individual tool handlers.
//!
//! No network is involved. We exercise the validation surface directly and
//! verify that required-field errors are returned consistently.

#![allow(dead_code)]

use serde_json::json;
use std::sync::Arc;

#[path = "../src/auth.rs"]
mod auth;
#[path = "../src/gcal.rs"]
mod gcal;
#[path = "../src/mcp.rs"]
mod mcp;
#[path = "../src/tools.rs"]
mod tools;

fn state() -> Arc<mcp::ServerState> {
    let tmp = std::env::temp_dir().join(format!(
        "gcal-mcp-test-tools-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    Arc::new(mcp::ServerState {
        http: reqwest::Client::new(),
        token_dir: tmp,
        credentials_path: None,
    })
}

fn is_invalid_params(err: &anyhow::Error) -> bool {
    format!("{err}").starts_with("invalid_params:")
}

#[tokio::test]
async fn test_list_events_requires_calendar_id() {
    let err = tools::call(state(), "list-events", json!({}))
        .await
        .expect_err("should fail without calendar_id");
    assert!(is_invalid_params(&err));
    assert!(format!("{err}").contains("calendar_id"));
}

#[tokio::test]
async fn test_create_event_requires_summary() {
    let err = tools::call(
        state(),
        "create-event",
        json!({
            "calendar_id": "primary",
            "start": "2026-05-14T10:00:00Z",
            "end": "2026-05-14T11:00:00Z"
        }),
    )
    .await
    .expect_err("should fail without summary");
    assert!(is_invalid_params(&err));
    assert!(format!("{err}").contains("summary"));
}

#[tokio::test]
async fn test_create_event_requires_start_and_end() {
    let err = tools::call(
        state(),
        "create-event",
        json!({
            "calendar_id": "primary",
            "summary": "Standup"
        }),
    )
    .await
    .expect_err("should fail without start/end");
    assert!(is_invalid_params(&err));
}

#[tokio::test]
async fn test_create_event_rejects_invalid_datetime() {
    let err = tools::call(
        state(),
        "create-event",
        json!({
            "calendar_id": "primary",
            "summary": "Bad",
            "start": "tomorrow at 9am",
            "end": "tomorrow at 10am"
        }),
    )
    .await
    .expect_err("should fail with non-RFC3339");
    assert!(is_invalid_params(&err));
}

#[tokio::test]
async fn test_respond_to_event_invalid_status() {
    let err = tools::call(
        state(),
        "respond-to-event",
        json!({
            "calendar_id": "primary",
            "event_id": "abc",
            "status": "maybe-ish"
        }),
    )
    .await
    .expect_err("should fail with invalid status");
    assert!(is_invalid_params(&err));
    assert!(format!("{err}").contains("status"));
}

#[tokio::test]
async fn test_get_current_time_returns_iso8601() {
    let value = tools::call(state(), "get-current-time", json!({}))
        .await
        .expect("get-current-time should always succeed");
    let iso = value["iso8601"].as_str().expect("iso8601 string");
    let parsed = chrono::DateTime::parse_from_rfc3339(iso);
    assert!(parsed.is_ok(), "iso8601 must be RFC3339-parseable: {iso}");
    assert_eq!(value["timezone"], "UTC");
    assert!(value["unix"].as_i64().unwrap_or(0) > 0);
}

#[tokio::test]
async fn test_manage_accounts_invalid_action() {
    let err = tools::call(
        state(),
        "manage-accounts",
        json!({ "action": "frobnicate" }),
    )
    .await
    .expect_err("unknown action should fail");
    assert!(is_invalid_params(&err));
}

#[tokio::test]
async fn test_manage_accounts_list_returns_array() {
    let value = tools::call(state(), "manage-accounts", json!({ "action": "list" }))
        .await
        .expect("list action should succeed");
    assert!(value["accounts"].is_array());
}

#[tokio::test]
async fn test_manage_accounts_remove_requires_name() {
    let err = tools::call(state(), "manage-accounts", json!({ "action": "remove" }))
        .await
        .expect_err("remove without name should fail");
    assert!(is_invalid_params(&err));
}

#[tokio::test]
async fn test_get_freebusy_requires_calendar_ids() {
    let err = tools::call(
        state(),
        "get-freebusy",
        json!({
            "time_min": "2026-05-14T00:00:00Z",
            "time_max": "2026-05-15T00:00:00Z"
        }),
    )
    .await
    .expect_err("should fail without calendar_ids");
    assert!(is_invalid_params(&err));
}

#[tokio::test]
async fn test_build_event_datetime_all_day() {
    let v = tools::build_event_datetime("2026-05-14", None).unwrap();
    assert_eq!(v["date"], "2026-05-14");
    assert!(v.get("dateTime").is_none());
}

#[tokio::test]
async fn test_build_event_datetime_with_timezone() {
    let v = tools::build_event_datetime("2026-05-14T10:00:00-05:00", Some("America/Lima"))
        .unwrap();
    assert_eq!(v["dateTime"], "2026-05-14T10:00:00-05:00");
    assert_eq!(v["timeZone"], "America/Lima");
}

#[tokio::test]
async fn test_unknown_tool_returns_invalid_params() {
    let err = tools::call(state(), "definitely-not-a-tool", json!({}))
        .await
        .expect_err("unknown tool should fail");
    assert!(is_invalid_params(&err));
}
