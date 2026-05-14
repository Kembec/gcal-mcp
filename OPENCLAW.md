# OpenClaw Compatibility

This MCP server is compatible with [OpenClaw](https://openclaw.ai) gateways.

## Configuration (openclaw.json)

```json
"google-calendar": {
  "command": "npx",
  "args": ["-y", "github:Kembec/gcal-mcp"],
  "env": {
    "GOOGLE_OAUTH_CREDENTIALS": "/home/node/.openclaw/credentials/gcp-oauth-calendar.json"
  }
}
```

## First-time auth

After adding to openclaw.json and restarting the gateway, ask the agent:
> "Add my Google account for Calendar"

The agent will call `manage-accounts` and provide an OAuth link to open in your browser.

## Security notes (vs upstream)

- CVEs fixed: `npm audit fix` applied (7 HIGH → 0)
- HTTP transport CSRF: `state` parameter now validated in `/oauth2callback`
- Upstream: [nspady/google-calendar-mcp](https://github.com/nspady/google-calendar-mcp)
