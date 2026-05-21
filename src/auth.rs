use anyhow::{anyhow, Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

pub const SCOPE: &str = "https://www.googleapis.com/auth/calendar";
pub const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredToken {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix seconds at which the access_token expires.
    pub expiry: i64,
    pub scope: Option<String>,
    pub token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthCredentialsFile {
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default)]
    client_secret: Option<String>,
    #[serde(default)]
    installed: Option<InstalledBlock>,
    #[serde(default)]
    web: Option<InstalledBlock>,
}

#[derive(Debug, Deserialize)]
struct InstalledBlock {
    client_id: String,
    client_secret: Option<String>,
}

pub struct OAuthCredentials {
    pub client_id: String,
    pub client_secret: Option<String>,
}

pub fn load_credentials(path: Option<&Path>) -> Result<OAuthCredentials> {
    let path = path.ok_or_else(|| {
        anyhow!("GOOGLE_OAUTH_CREDENTIALS env var is not set (path to OAuth client JSON)")
    })?;
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read credentials file {}", path.display()))?;
    let parsed: OAuthCredentialsFile = serde_json::from_str(&raw)
        .with_context(|| format!("parse credentials JSON {}", path.display()))?;

    if let Some(b) = parsed.installed.or(parsed.web) {
        return Ok(OAuthCredentials {
            client_id: b.client_id,
            client_secret: b.client_secret,
        });
    }

    let client_id = parsed
        .client_id
        .ok_or_else(|| anyhow!("credentials file missing client_id"))?;
    Ok(OAuthCredentials {
        client_id,
        client_secret: parsed.client_secret,
    })
}

pub fn token_path(token_dir: &Path, account: &str) -> PathBuf {
    let safe = account
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '@' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    token_dir.join(format!("{}.json", safe))
}

pub fn load_token(token_dir: &Path, account: &str) -> Result<Option<StoredToken>> {
    let path = token_path(token_dir, account);
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read token file {}", path.display()))?;
    let token: StoredToken = serde_json::from_str(&raw)
        .with_context(|| format!("parse token JSON {}", path.display()))?;
    Ok(Some(token))
}

pub fn save_token(token_dir: &Path, account: &str, token: &StoredToken) -> Result<()> {
    std::fs::create_dir_all(token_dir).ok();
    let path = token_path(token_dir, account);
    let raw = serde_json::to_string_pretty(token)?;
    std::fs::write(&path, raw).with_context(|| format!("write token file {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn list_accounts(token_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(token_dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                out.push(stem.to_string());
            }
        }
    }
    out.sort();
    out
}

pub fn remove_account(token_dir: &Path, account: &str) -> Result<()> {
    let path = token_path(token_dir, account);
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("remove token file {}", path.display()))?;
    }
    Ok(())
}

pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn pkce_pair() -> (String, String) {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let verifier_bytes: [u8; 32] = rand_bytes();
    let verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
    let challenge = sha256_b64url(verifier.as_bytes());
    (verifier, challenge)
}

fn rand_bytes() -> [u8; 32] {
    let mut buf = [0u8; 32];
    let a = uuid::Uuid::new_v4();
    let b = uuid::Uuid::new_v4();
    buf[..16].copy_from_slice(a.as_bytes());
    buf[16..].copy_from_slice(b.as_bytes());
    buf
}

fn sha256_b64url(input: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    URL_SAFE_NO_PAD.encode(sha256(input))
}

/// Minimal SHA-256 implementation so we do not pull in another crate.
fn sha256(message: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (message.len() as u64) * 8;
    let mut data = message.to_vec();
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in data.chunks(64) {
        let mut w = [0u32; 64];
        for (i, item) in w.iter_mut().enumerate().take(16) {
            let j = i * 4;
            *item = u32::from_be_bytes([chunk[j], chunk[j + 1], chunk[j + 2], chunk[j + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let mj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(mj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
}

pub async fn refresh_access_token(
    http: &reqwest::Client,
    creds: &OAuthCredentials,
    refresh_token: &str,
) -> Result<TokenResponse> {
    let mut form = vec![
        ("client_id", creds.client_id.as_str()),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];
    if let Some(s) = creds.client_secret.as_deref() {
        form.push(("client_secret", s));
    }
    let resp = http
        .post(TOKEN_URL)
        .form(&form)
        .send()
        .await
        .context("token refresh request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("token refresh failed ({status}): {body}"));
    }
    let parsed: TokenResponse = resp.json().await.context("parse refresh response")?;
    Ok(parsed)
}

pub async fn exchange_code(
    http: &reqwest::Client,
    creds: &OAuthCredentials,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse> {
    let mut form = vec![
        ("client_id", creds.client_id.as_str()),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", verifier),
    ];
    if let Some(s) = creds.client_secret.as_deref() {
        form.push(("client_secret", s));
    }
    let resp = http
        .post(TOKEN_URL)
        .form(&form)
        .send()
        .await
        .context("token exchange request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("token exchange failed ({status}): {body}"));
    }
    let parsed: TokenResponse = resp.json().await.context("parse token response")?;
    Ok(parsed)
}

fn find_free_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

pub async fn interactive_login(
    http: &reqwest::Client,
    creds: &OAuthCredentials,
) -> Result<StoredToken> {
    let port = find_free_port()?;
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    let (verifier, challenge) = pkce_pair();
    let state_param = uuid::Uuid::new_v4().to_string();

    let mut authorize = url::Url::parse(AUTH_URL)?;
    authorize
        .query_pairs_mut()
        .append_pair("client_id", &creds.client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", SCOPE)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", &state_param);

    let url_string = authorize.to_string();
    eprintln!("Open this URL in a browser to authorize gcal-mcp:");
    eprintln!("{url_string}");
    let _ = open::that(url_string);

    let (tx, rx) = mpsc::channel::<Result<String, String>>();
    let expected_state = state_param.clone();
    let bind_addr = format!("127.0.0.1:{port}");
    std::thread::spawn(move || {
        let server = match tiny_http::Server::http(&bind_addr) {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(Err(format!("failed to start local server: {e}")));
                return;
            }
        };
        for request in server.incoming_requests() {
            let url_str = format!("http://localhost{}", request.url());
            let parsed = url::Url::parse(&url_str);
            let mut code = None;
            let mut state_in = None;
            let mut error = None;
            if let Ok(u) = parsed {
                for (k, v) in u.query_pairs() {
                    match k.as_ref() {
                        "code" => code = Some(v.into_owned()),
                        "state" => state_in = Some(v.into_owned()),
                        "error" => error = Some(v.into_owned()),
                        _ => {}
                    }
                }
            }
            let response_body = if let Some(err) = error.as_ref() {
                format!("Authorization failed: {err}. You may close this tab.")
            } else if code.is_some() && state_in.as_deref() == Some(expected_state.as_str()) {
                "Authorization complete. You may close this tab.".to_string()
            } else {
                "Invalid response. You may close this tab.".to_string()
            };
            let response = tiny_http::Response::from_string(response_body).with_header(
                "Content-Type: text/plain"
                    .parse::<tiny_http::Header>()
                    .unwrap(),
            );
            let _ = request.respond(response);

            if let Some(err) = error {
                let _ = tx.send(Err(err));
                return;
            }
            if let (Some(c), Some(s)) = (code, state_in) {
                if s != expected_state {
                    let _ = tx.send(Err("state mismatch in OAuth callback".to_string()));
                    return;
                }
                let _ = tx.send(Ok(c));
                return;
            }
        }
    });

    let code = tokio::task::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(300))
            .map_err(|e| format!("waiting for OAuth callback: {e}"))?
    })
    .await
    .map_err(|e| anyhow!("oauth callback task panicked: {e}"))?
    .map_err(|e| anyhow!("{e}"))?;

    let tok = exchange_code(http, creds, &code, &verifier, &redirect_uri).await?;
    let refresh = tok
        .refresh_token
        .ok_or_else(|| anyhow!("Google did not return a refresh_token"))?;
    let expiry = now_unix() + tok.expires_in.unwrap_or(3600);
    Ok(StoredToken {
        access_token: tok.access_token,
        refresh_token: refresh,
        expiry,
        scope: tok.scope,
        token_type: tok.token_type,
    })
}

pub async fn get_token(
    http: &reqwest::Client,
    token_dir: &Path,
    credentials_path: Option<&Path>,
    account: &str,
) -> Result<String> {
    let creds = load_credentials(credentials_path)?;
    if let Some(mut existing) = load_token(token_dir, account)? {
        if existing.expiry > now_unix() + 60 {
            return Ok(existing.access_token);
        }
        // Try refresh first.
        let refreshed = refresh_access_token(http, &creds, &existing.refresh_token).await?;
        existing.access_token = refreshed.access_token.clone();
        if let Some(rt) = refreshed.refresh_token {
            existing.refresh_token = rt;
        }
        existing.expiry = now_unix() + refreshed.expires_in.unwrap_or(3600);
        if refreshed.scope.is_some() {
            existing.scope = refreshed.scope;
        }
        if refreshed.token_type.is_some() {
            existing.token_type = refreshed.token_type;
        }
        save_token(token_dir, account, &existing)?;
        return Ok(existing.access_token);
    }

    // No stored token — run the interactive flow.
    let fresh = interactive_login(http, &creds).await?;
    let access = fresh.access_token.clone();
    save_token(token_dir, account, &fresh)?;
    Ok(access)
}
