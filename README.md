# gcal-mcp

Fork of [nspady/google-calendar-mcp](https://github.com/nspady/google-calendar-mcp) (`@cocal/google-calendar-mcp`) with security patches and OpenClaw compatibility. The original design, implementation, and multi-account OAuth flow are by the upstream author — all credit is his.

This fork adds: 7 CVEs resolved (`npm audit fix`), CSRF fix in the HTTP transport, and a ready-to-use OpenClaw config block. See [SECURITY_AUDIT.md](./SECURITY_AUDIT.md) and [OPENCLAW.md](./OPENCLAW.md).

## What it does

Exposes Google Calendar as MCP tools: read events, create, update, delete, respond to invitations, check availability. Supports multiple accounts in parallel — useful if you manage work and personal separately.

## Installation

You need an OAuth 2.0 credentials file (Desktop App type) from Google Cloud Console with the Calendar API enabled.

```bash
npx @kembec/gcal-mcp
```

With Claude Desktop:

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

On startup the server opens a browser for the OAuth flow. Once authorized, the token is saved locally — no need to repeat it. To add a second account: ask Claude to use the `manage-accounts` tool.

If the token expires (Google test mode, 7 days), re-authorize with:

```bash
npx @kembec/gcal-mcp auth
```

## Available tools

`list-calendars`, `list-events`, `search-events`, `get-event`, `create-event`, `update-event`, `delete-event`, `respond-to-event`, `get-freebusy`, `get-current-time`, `manage-accounts`

For OpenClaw see [OPENCLAW.md](./OPENCLAW.md).

## Credits

Original project: [nspady/google-calendar-mcp](https://github.com/nspady/google-calendar-mcp) by [@nspady](https://github.com/nspady). Published on npm as [`@cocal/google-calendar-mcp`](https://www.npmjs.com/package/@cocal/google-calendar-mcp).

## License

MIT
