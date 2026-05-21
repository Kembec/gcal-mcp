use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

mod auth;
mod gcal;
mod mcp;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.get(1).map(|s| s.as_str()) == Some("auth") {
        return cmd_auth().await;
    }

    eprintln!("gcal-mcp starting");

    let state = Arc::new(mcp::ServerState::new()?);

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = reader.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        if let Some(response) = mcp::handle_line(state.clone(), &line).await {
            stdout.write_all(response.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
    }

    Ok(())
}

async fn cmd_auth() -> Result<()> {
    let state = mcp::ServerState::new()?;
    let account = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "default".to_string());
    eprintln!("Authenticating account '{account}'...");
    let token = auth::get_token(
        &state.http,
        &state.token_dir,
        state.credentials_path.as_deref(),
        &account,
    )
    .await?;
    eprintln!("Authentication successful. Token saved.");
    eprintln!(
        "Access token (first 20 chars): {}...",
        &token[..token.len().min(20)]
    );
    Ok(())
}
