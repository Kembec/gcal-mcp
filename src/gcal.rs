use anyhow::{anyhow, Context, Result};
use reqwest::Method;
use serde_json::{json, Value};

pub const DEFAULT_BASE_URL: &str = "https://www.googleapis.com/calendar/v3";

pub struct CalendarClient {
    pub client: reqwest::Client,
    pub token: String,
    pub base_url: String,
}

impl CalendarClient {
    pub fn new(client: reqwest::Client, token: String) -> Self {
        Self {
            client,
            token,
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&Value>,
    ) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self
            .client
            .request(method.clone(), &url)
            .bearer_auth(&self.token);
        if !query.is_empty() {
            req = req.query(query);
        }
        if let Some(b) = body {
            req = req.json(b);
        }
        let resp = req
            .send()
            .await
            .with_context(|| format!("HTTP {method} {url}"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!(
                "Google Calendar API error ({status}) on {method} {url}: {text}"
            ));
        }
        if text.is_empty() {
            return Ok(json!({ "ok": true }));
        }
        serde_json::from_str(&text).with_context(|| format!("parse JSON from {url}"))
    }

    pub async fn list_calendars(&self) -> Result<Value> {
        self.request(Method::GET, "/users/me/calendarList", &[], None)
            .await
    }

    pub async fn list_events(
        &self,
        calendar_id: &str,
        time_min: Option<&str>,
        time_max: Option<&str>,
        max_results: Option<i64>,
        page_token: Option<&str>,
    ) -> Result<Value> {
        let mut query: Vec<(&str, String)> = Vec::new();
        query.push(("singleEvents", "true".to_string()));
        query.push(("orderBy", "startTime".to_string()));
        if let Some(t) = time_min {
            query.push(("timeMin", t.to_string()));
        }
        if let Some(t) = time_max {
            query.push(("timeMax", t.to_string()));
        }
        if let Some(n) = max_results {
            query.push(("maxResults", n.to_string()));
        }
        if let Some(tok) = page_token {
            query.push(("pageToken", tok.to_string()));
        }
        let path = format!("/calendars/{}/events", percent_encode(calendar_id));
        self.request(Method::GET, &path, &query, None).await
    }

    pub async fn get_event(&self, calendar_id: &str, event_id: &str) -> Result<Value> {
        let path = format!(
            "/calendars/{}/events/{}",
            percent_encode(calendar_id),
            percent_encode(event_id)
        );
        self.request(Method::GET, &path, &[], None).await
    }

    pub async fn search_events(
        &self,
        calendar_id: &str,
        query: &str,
        time_min: Option<&str>,
        time_max: Option<&str>,
    ) -> Result<Value> {
        let mut q: Vec<(&str, String)> = vec![
            ("q", query.to_string()),
            ("singleEvents", "true".to_string()),
            ("orderBy", "startTime".to_string()),
        ];
        if let Some(t) = time_min {
            q.push(("timeMin", t.to_string()));
        }
        if let Some(t) = time_max {
            q.push(("timeMax", t.to_string()));
        }
        let path = format!("/calendars/{}/events", percent_encode(calendar_id));
        self.request(Method::GET, &path, &q, None).await
    }

    pub async fn create_event(&self, calendar_id: &str, body: &Value) -> Result<Value> {
        let path = format!("/calendars/{}/events", percent_encode(calendar_id));
        self.request(Method::POST, &path, &[], Some(body)).await
    }

    pub async fn update_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        body: &Value,
    ) -> Result<Value> {
        let path = format!(
            "/calendars/{}/events/{}",
            percent_encode(calendar_id),
            percent_encode(event_id)
        );
        self.request(Method::PUT, &path, &[], Some(body)).await
    }

    pub async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<Value> {
        let path = format!(
            "/calendars/{}/events/{}",
            percent_encode(calendar_id),
            percent_encode(event_id)
        );
        self.request(Method::DELETE, &path, &[], None).await
    }

    pub async fn respond_to_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        status: &str,
    ) -> Result<Value> {
        let body = json!({
            "attendees": [
                { "self": true, "responseStatus": status }
            ]
        });
        let path = format!(
            "/calendars/{}/events/{}",
            percent_encode(calendar_id),
            percent_encode(event_id)
        );
        self.request(Method::PATCH, &path, &[], Some(&body)).await
    }

    pub async fn get_freebusy(
        &self,
        time_min: &str,
        time_max: &str,
        calendar_ids: Vec<String>,
    ) -> Result<Value> {
        let items: Vec<Value> = calendar_ids
            .into_iter()
            .map(|id| json!({ "id": id }))
            .collect();
        let body = json!({
            "timeMin": time_min,
            "timeMax": time_max,
            "items": items
        });
        self.request(Method::POST, "/freeBusy", &[], Some(&body))
            .await
    }
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        let c = *byte;
        let unreserved = c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.' | b'~');
        if unreserved {
            out.push(c as char);
        } else {
            out.push_str(&format!("%{:02X}", c));
        }
    }
    out
}
