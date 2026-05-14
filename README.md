# gcal-mcp

MCP server for Google Calendar. Fork of `@cocal/google-calendar-mcp` with security patches and OpenClaw support.

This fork adds: 7 CVEs fixed, CSRF patch on the HTTP transport, and a ready-to-use OpenClaw config. See [SECURITY_AUDIT.md](./SECURITY_AUDIT.md) and [OPENCLAW.md](./OPENCLAW.md).

## Installation

Requires an OAuth 2.0 credentials file (Desktop App type) from Google Cloud Console with Calendar API enabled.

```bash
npx @kembec/gcal-mcp
```

Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "google-calendar": {
      "command": "npx",
      "args": ["@kembec/gcal-mcp"],
      "env": {
        "GOOGLE_OAUTH_CREDENTIALS": "/path/to/gcp-oauth.keys.json"
      }
    }
  }
}
```

## First run

On startup the server opens a browser for the OAuth flow. Token is saved locally — no need to repeat it. To add a second account ask Claude to use the `manage-accounts` tool.

If the token expires (Google test mode expires after 7 days):

```bash
npx @kembec/gcal-mcp auth
```

## Tools

`list-calendars` · `list-events` · `search-events` · `get-event` · `create-event` · `update-event` · `delete-event` · `respond-to-event` · `get-freebusy` · `get-current-time` · `manage-accounts`

For OpenClaw see [OPENCLAW.md](./OPENCLAW.md).

## License

MIT

## Credits

Original project: [nspady/google-calendar-mcp](https://github.com/nspady/google-calendar-mcp) by [@nspady](https://github.com/nspady), published as [`@cocal/google-calendar-mcp`](https://www.npmjs.com/package/@cocal/google-calendar-mcp). All credit for the design, implementation, and multi-account OAuth flow goes to the upstream author.
