# gcal-mcp

An MCP server that exposes Google Calendar as tools for any model that speaks the Model Context Protocol.

The binary talks JSON-RPC 2.0 over stdio, the same way language servers and other MCP servers do. It handles the Google OAuth2 PKCE flow itself — point it at a downloaded OAuth client JSON, run it once, and tokens are cached under `~/.config/kembec/gcal-mcp/tokens/`. The server stays small (a single Rust binary, no daemon) and reuses one refresh token per account.

## Install

```sh
npm install -g @kembec/gcal-mcp
```

The umbrella package only ships a thin Node launcher; the actual binary comes from the matching `@kembec/gcal-mcp-<platform>` optional dependency that npm picks at install time. Supported targets: `darwin-arm64`, `darwin-x64`, `linux-x64`, `win32-x64`.

You can also build from source:

```sh
git clone <this-repo>
cd gcal-mcp
cargo build --release
# binary at target/release/gcal-mcp
```

## Configure

1. Create an OAuth client in the Google Cloud Console. Choose "Desktop app". Download the resulting JSON.
2. Point the server at it:

   ```sh
   export GOOGLE_OAUTH_CREDENTIALS=/path/to/client_secret.json
   ```

3. On the first tool call that needs the API, the server opens a browser, waits for the callback on `http://127.0.0.1:8080/callback`, and writes a token file. Subsequent runs refresh silently.

The Calendar scope used is `https://www.googleapis.com/auth/calendar`.

## Use from an MCP client

Add the binary to your client config (Claude Desktop, OpenClaw, etc.) as a stdio MCP server:

```json
{
  "command": "gcal-mcp",
  "env": {
    "GOOGLE_OAUTH_CREDENTIALS": "/path/to/client_secret.json"
  }
}
```

The server advertises 11 tools: `list-calendars`, `list-events`, `search-events`, `get-event`, `create-event`, `update-event`, `delete-event`, `respond-to-event`, `get-freebusy`, `get-current-time`, and `manage-accounts`. Each one takes a JSON `arguments` object; required fields are validated up front and reported as `-32602` errors. `manage-accounts` is the entry point for adding or removing additional Google accounts — most other tools accept an optional `account` field to pick which stored token to use.

Datetimes are RFC3339 (e.g. `2026-05-14T10:00:00-05:00`). Pass `YYYY-MM-DD` to `start`/`end` of `create-event` to make an all-day event. Pass an IANA `timezone` to attach a `timeZone` to dated events.

## License

MIT.
