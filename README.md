# gcal-mcp

MCP server for Google Calendar. Single Rust binary, no daemon, handles OAuth2 PKCE itself — point it at a credentials JSON, authorize once, tokens refresh silently.

[![npm](https://img.shields.io/npm/v/@kembec/gcal-mcp)](https://www.npmjs.com/package/@kembec/gcal-mcp)
[![license](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)](#)

## Install

```sh
npm install -g @kembec/gcal-mcp
```

Supported targets: `darwin-arm64`, `darwin-x64`, `linux-x64`, `win32-x64`.

Or build from source:

```sh
git clone https://github.com/Kembec/gcal-mcp.git
cd gcal-mcp
cargo build --release
```

## Configure

1. Create an OAuth client in the [Google Cloud Console](https://console.cloud.google.com). Choose **Desktop app**. Download the JSON.
2. Set the env var pointing to it:

```sh
export GOOGLE_OAUTH_CREDENTIALS=/path/to/client_secret.json
```

3. On the first tool call the server opens a browser for authorization, catches the callback on a random local port, and writes a token under `~/.config/kembec/gcal-mcp/tokens/`. Subsequent runs refresh silently.

The Calendar scope used is `https://www.googleapis.com/auth/calendar`.

## Add to your MCP client

### Cursor / Claude Desktop

```json
{
  "mcpServers": {
    "gcal": {
      "command": "gcal-mcp",
      "env": {
        "GOOGLE_OAUTH_CREDENTIALS": "/path/to/client_secret.json"
      }
    }
  }
}
```

Or with `npx`:

```json
{
  "mcpServers": {
    "gcal": {
      "command": "npx",
      "args": ["-y", "@kembec/gcal-mcp"],
      "env": {
        "GOOGLE_OAUTH_CREDENTIALS": "/path/to/client_secret.json"
      }
    }
  }
}
```

## Tools

| Tool | Description |
|------|-------------|
| `list-calendars` | List all calendars on the account |
| `list-events` | List events from a calendar with optional time window |
| `search-events` | Full-text search over events |
| `get-event` | Fetch a single event by id |
| `create-event` | Create an event (RFC3339 or `YYYY-MM-DD` for all-day) |
| `update-event` | Patch an existing event |
| `delete-event` | Delete an event |
| `respond-to-event` | Accept / decline / tentative an invitation |
| `get-freebusy` | Query free/busy windows across calendars |
| `get-current-time` | Return current UTC time |
| `manage-accounts` | Add, list, or remove stored OAuth accounts |

All tools accept an optional `account` field to select which stored token to use (defaults to `"default"`).

## Multi-account

```json
{ "name": "manage-accounts", "arguments": { "action": "add", "account_name": "work" } }
```

Then pass `"account": "work"` to any tool.

## Token storage

Tokens live in `~/.config/kembec/gcal-mcp/tokens/<account>.json` with `0600` permissions (Unix). The directory is `0700`.

## License

MIT.
