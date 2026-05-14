use anyhow::{anyhow, Result};
use chrono::{SecondsFormat, Utc};
use serde_json::{json, Map, Value};
use std::sync::Arc;

use crate::auth;
use crate::gcal::CalendarClient;
use crate::mcp::ServerState;

pub const DEFAULT_ACCOUNT: &str = "default";
pub const VALID_RESPONSE_STATUS: &[&str] = &["accepted", "declined", "tentative", "needsAction"];

pub fn tools_list() -> Value {
    json!([
        {
            "name": "list-calendars",
            "description": "List calendars on the authenticated Google account, including primary and shared calendars.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "account": { "type": "string", "description": "Account name to use (defaults to 'default')." }
                },
                "additionalProperties": false
            }
        },
        {
            "name": "list-events",
            "description": "List events from a calendar within an optional time window.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "calendar_id": { "type": "string", "description": "Calendar identifier ('primary' or one returned by list-calendars)." },
                    "time_min": { "type": "string", "description": "Lower bound (RFC3339)." },
                    "time_max": { "type": "string", "description": "Upper bound (RFC3339)." },
                    "max_results": { "type": "integer", "minimum": 1, "maximum": 2500, "default": 250 },
                    "page_token": { "type": "string" },
                    "account": { "type": "string" }
                },
                "required": ["calendar_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "search-events",
            "description": "Full-text search over event summaries, descriptions and attendees.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "calendar_id": { "type": "string" },
                    "query": { "type": "string" },
                    "time_min": { "type": "string" },
                    "time_max": { "type": "string" },
                    "account": { "type": "string" }
                },
                "required": ["calendar_id", "query"],
                "additionalProperties": false
            }
        },
        {
            "name": "get-event",
            "description": "Fetch a single event by id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "calendar_id": { "type": "string" },
                    "event_id": { "type": "string" },
                    "account": { "type": "string" }
                },
                "required": ["calendar_id", "event_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "create-event",
            "description": "Create a new event. start and end accept RFC3339 datetimes or YYYY-MM-DD (all-day).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "calendar_id": { "type": "string" },
                    "summary": { "type": "string" },
                    "start": { "type": "string", "description": "RFC3339 datetime or YYYY-MM-DD for all-day events." },
                    "end": { "type": "string" },
                    "description": { "type": "string" },
                    "location": { "type": "string" },
                    "attendees": {
                        "type": "array",
                        "items": { "type": "string", "description": "Email address." }
                    },
                    "recurrence": {
                        "type": "array",
                        "items": { "type": "string", "description": "RRULE/RDATE/EXDATE lines." }
                    },
                    "timezone": { "type": "string", "description": "IANA timezone, e.g. 'America/Lima'." },
                    "account": { "type": "string" }
                },
                "required": ["calendar_id", "summary", "start", "end"],
                "additionalProperties": false
            }
        },
        {
            "name": "update-event",
            "description": "Patch an existing event. Only provided fields are changed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "calendar_id": { "type": "string" },
                    "event_id": { "type": "string" },
                    "summary": { "type": "string" },
                    "description": { "type": "string" },
                    "start": { "type": "string" },
                    "end": { "type": "string" },
                    "location": { "type": "string" },
                    "attendees": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "recurrence": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "timezone": { "type": "string" },
                    "account": { "type": "string" }
                },
                "required": ["calendar_id", "event_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "delete-event",
            "description": "Delete an event.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "calendar_id": { "type": "string" },
                    "event_id": { "type": "string" },
                    "account": { "type": "string" }
                },
                "required": ["calendar_id", "event_id"],
                "additionalProperties": false
            }
        },
        {
            "name": "respond-to-event",
            "description": "Reply to an invitation as the authenticated user.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "calendar_id": { "type": "string" },
                    "event_id": { "type": "string" },
                    "status": {
                        "type": "string",
                        "enum": ["accepted", "declined", "tentative", "needsAction"]
                    },
                    "account": { "type": "string" }
                },
                "required": ["calendar_id", "event_id", "status"],
                "additionalProperties": false
            }
        },
        {
            "name": "get-freebusy",
            "description": "Query free/busy windows across one or more calendars.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "time_min": { "type": "string" },
                    "time_max": { "type": "string" },
                    "calendar_ids": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "account": { "type": "string" }
                },
                "required": ["time_min", "time_max", "calendar_ids"],
                "additionalProperties": false
            }
        },
        {
            "name": "get-current-time",
            "description": "Return the current UTC time as an ISO 8601 string.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "manage-accounts",
            "description": "List, add or remove stored OAuth accounts.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["list", "add", "remove"] },
                    "account_name": { "type": "string" }
                },
                "required": ["action"],
                "additionalProperties": false
            }
        }
    ])
}

pub async fn call(state: Arc<ServerState>, name: &str, args: Value) -> Result<Value> {
    match name {
        "list-calendars" => list_calendars(state, args).await,
        "list-events" => list_events(state, args).await,
        "search-events" => search_events(state, args).await,
        "get-event" => get_event(state, args).await,
        "create-event" => create_event(state, args).await,
        "update-event" => update_event(state, args).await,
        "delete-event" => delete_event(state, args).await,
        "respond-to-event" => respond_to_event(state, args).await,
        "get-freebusy" => get_freebusy(state, args).await,
        "get-current-time" => get_current_time(args),
        "manage-accounts" => manage_accounts(state, args).await,
        _ => Err(invalid_params(format!("unknown tool: {name}"))),
    }
}

// ---------- Validation helpers ----------

pub fn invalid_params(msg: impl Into<String>) -> anyhow::Error {
    anyhow!("invalid_params: {}", msg.into())
}

fn require_str<'a>(args: &'a Value, field: &str) -> Result<&'a str> {
    let v = args
        .get(field)
        .ok_or_else(|| invalid_params(format!("missing required field '{field}'")))?;
    let s = v
        .as_str()
        .ok_or_else(|| invalid_params(format!("field '{field}' must be a string")))?;
    if s.is_empty() {
        return Err(invalid_params(format!("field '{field}' must not be empty")));
    }
    Ok(s)
}

fn opt_str<'a>(args: &'a Value, field: &str) -> Result<Option<&'a str>> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) if !s.is_empty() => Ok(Some(s.as_str())),
        Some(Value::String(_)) => Ok(None),
        _ => Err(invalid_params(format!("field '{field}' must be a string"))),
    }
}

fn opt_i64(args: &Value, field: &str) -> Result<Option<i64>> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n
            .as_i64()
            .ok_or_else(|| invalid_params(format!("field '{field}' must be an integer")))
            .map(Some),
        _ => Err(invalid_params(format!("field '{field}' must be an integer"))),
    }
}

fn opt_str_array(args: &Value, field: &str) -> Result<Option<Vec<String>>> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(arr)) => {
            let mut out = Vec::with_capacity(arr.len());
            for (i, v) in arr.iter().enumerate() {
                match v.as_str() {
                    Some(s) => out.push(s.to_string()),
                    None => {
                        return Err(invalid_params(format!(
                            "field '{field}[{i}]' must be a string"
                        )))
                    }
                }
            }
            Ok(Some(out))
        }
        _ => Err(invalid_params(format!("field '{field}' must be an array"))),
    }
}

fn require_str_array(args: &Value, field: &str) -> Result<Vec<String>> {
    let arr = opt_str_array(args, field)?
        .ok_or_else(|| invalid_params(format!("missing required field '{field}'")))?;
    if arr.is_empty() {
        return Err(invalid_params(format!("field '{field}' must not be empty")));
    }
    Ok(arr)
}

fn account_name<'a>(args: &'a Value) -> &'a str {
    args.get("account")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_ACCOUNT)
}

/// Build a Google Calendar EventDateTime block.
/// Accepts either a `YYYY-MM-DD` (all-day) or an RFC3339 datetime.
pub fn build_event_datetime(raw: &str, timezone: Option<&str>) -> Result<Value> {
    if raw.len() == 10 && raw.chars().filter(|c| *c == '-').count() == 2 {
        // All-day; basic format validation: YYYY-MM-DD
        if chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d").is_err() {
            return Err(invalid_params(format!("invalid date '{raw}'")));
        }
        return Ok(json!({ "date": raw }));
    }
    if chrono::DateTime::parse_from_rfc3339(raw).is_err() {
        return Err(invalid_params(format!(
            "invalid datetime '{raw}', expected RFC3339"
        )));
    }
    let mut block = Map::new();
    block.insert("dateTime".to_string(), Value::String(raw.to_string()));
    if let Some(tz) = timezone {
        block.insert("timeZone".to_string(), Value::String(tz.to_string()));
    }
    Ok(Value::Object(block))
}

pub fn build_event_body(args: &Value, include_required: bool) -> Result<Value> {
    let timezone = opt_str(args, "timezone")?;
    let mut body = Map::new();

    if let Some(summary) = opt_str(args, "summary")? {
        body.insert("summary".to_string(), Value::String(summary.to_string()));
    } else if include_required {
        return Err(invalid_params("missing required field 'summary'"));
    }

    if let Some(start) = opt_str(args, "start")? {
        body.insert("start".to_string(), build_event_datetime(start, timezone)?);
    } else if include_required {
        return Err(invalid_params("missing required field 'start'"));
    }

    if let Some(end) = opt_str(args, "end")? {
        body.insert("end".to_string(), build_event_datetime(end, timezone)?);
    } else if include_required {
        return Err(invalid_params("missing required field 'end'"));
    }

    if let Some(desc) = opt_str(args, "description")? {
        body.insert("description".to_string(), Value::String(desc.to_string()));
    }
    if let Some(loc) = opt_str(args, "location")? {
        body.insert("location".to_string(), Value::String(loc.to_string()));
    }
    if let Some(attendees) = opt_str_array(args, "attendees")? {
        let attendees_json: Vec<Value> = attendees
            .into_iter()
            .map(|email| json!({ "email": email }))
            .collect();
        body.insert("attendees".to_string(), Value::Array(attendees_json));
    }
    if let Some(rec) = opt_str_array(args, "recurrence")? {
        body.insert(
            "recurrence".to_string(),
            Value::Array(rec.into_iter().map(Value::String).collect()),
        );
    }

    Ok(Value::Object(body))
}

// ---------- Tool implementations ----------

async fn token_for(state: &ServerState, account: &str) -> Result<String> {
    auth::get_token(
        &state.http,
        &state.token_dir,
        state.credentials_path.as_deref(),
        account,
    )
    .await
}

async fn list_calendars(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let account = account_name(&args);
    let token = token_for(&state, account).await?;
    let client = CalendarClient::new(state.http.clone(), token);
    client.list_calendars().await
}

async fn list_events(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let calendar_id = require_str(&args, "calendar_id")?;
    let time_min = opt_str(&args, "time_min")?;
    let time_max = opt_str(&args, "time_max")?;
    let max_results = opt_i64(&args, "max_results")?.or(Some(250));
    let page_token = opt_str(&args, "page_token")?;
    let account = account_name(&args);

    let token = token_for(&state, account).await?;
    let client = CalendarClient::new(state.http.clone(), token);
    client
        .list_events(calendar_id, time_min, time_max, max_results, page_token)
        .await
}

async fn search_events(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let calendar_id = require_str(&args, "calendar_id")?;
    let query = require_str(&args, "query")?;
    let time_min = opt_str(&args, "time_min")?;
    let time_max = opt_str(&args, "time_max")?;
    let account = account_name(&args);

    let token = token_for(&state, account).await?;
    let client = CalendarClient::new(state.http.clone(), token);
    client
        .search_events(calendar_id, query, time_min, time_max)
        .await
}

async fn get_event(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let calendar_id = require_str(&args, "calendar_id")?;
    let event_id = require_str(&args, "event_id")?;
    let account = account_name(&args);

    let token = token_for(&state, account).await?;
    let client = CalendarClient::new(state.http.clone(), token);
    client.get_event(calendar_id, event_id).await
}

async fn create_event(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let calendar_id = require_str(&args, "calendar_id")?;
    let body = build_event_body(&args, true)?;
    let account = account_name(&args);

    let token = token_for(&state, account).await?;
    let client = CalendarClient::new(state.http.clone(), token);
    client.create_event(calendar_id, &body).await
}

async fn update_event(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let calendar_id = require_str(&args, "calendar_id")?;
    let event_id = require_str(&args, "event_id")?;
    let body = build_event_body(&args, false)?;
    if body.as_object().map(|m| m.is_empty()).unwrap_or(true) {
        return Err(invalid_params(
            "update-event needs at least one field to change",
        ));
    }
    let account = account_name(&args);

    let token = token_for(&state, account).await?;
    let client = CalendarClient::new(state.http.clone(), token);
    client.update_event(calendar_id, event_id, &body).await
}

async fn delete_event(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let calendar_id = require_str(&args, "calendar_id")?;
    let event_id = require_str(&args, "event_id")?;
    let account = account_name(&args);

    let token = token_for(&state, account).await?;
    let client = CalendarClient::new(state.http.clone(), token);
    client.delete_event(calendar_id, event_id).await
}

async fn respond_to_event(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let calendar_id = require_str(&args, "calendar_id")?;
    let event_id = require_str(&args, "event_id")?;
    let status = require_str(&args, "status")?;
    if !VALID_RESPONSE_STATUS.contains(&status) {
        return Err(invalid_params(format!(
            "status must be one of {:?}, got '{status}'",
            VALID_RESPONSE_STATUS
        )));
    }
    let account = account_name(&args);

    let token = token_for(&state, account).await?;
    let client = CalendarClient::new(state.http.clone(), token);
    client.respond_to_event(calendar_id, event_id, status).await
}

async fn get_freebusy(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let time_min = require_str(&args, "time_min")?;
    let time_max = require_str(&args, "time_max")?;
    let calendar_ids = require_str_array(&args, "calendar_ids")?;
    let account = account_name(&args);

    let token = token_for(&state, account).await?;
    let client = CalendarClient::new(state.http.clone(), token);
    client.get_freebusy(time_min, time_max, calendar_ids).await
}

pub fn get_current_time(_args: Value) -> Result<Value> {
    let now = Utc::now();
    Ok(json!({
        "iso8601": now.to_rfc3339_opts(SecondsFormat::Secs, true),
        "unix": now.timestamp(),
        "timezone": "UTC"
    }))
}

async fn manage_accounts(state: Arc<ServerState>, args: Value) -> Result<Value> {
    let action = require_str(&args, "action")?;
    match action {
        "list" => {
            let accounts = auth::list_accounts(&state.token_dir);
            Ok(json!({ "accounts": accounts }))
        }
        "add" => {
            let account = args
                .get("account_name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(DEFAULT_ACCOUNT);
            let creds = auth::load_credentials(state.credentials_path.as_deref())?;
            let token = auth::interactive_login(&state.http, &creds).await?;
            auth::save_token(&state.token_dir, account, &token)?;
            Ok(json!({
                "ok": true,
                "account": account,
                "scopes": token.scope
            }))
        }
        "remove" => {
            let account = args
                .get("account_name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| invalid_params("'remove' requires 'account_name'"))?;
            auth::remove_account(&state.token_dir, account)?;
            Ok(json!({ "ok": true, "removed": account }))
        }
        other => Err(invalid_params(format!(
            "action must be one of list/add/remove, got '{other}'"
        ))),
    }
}
