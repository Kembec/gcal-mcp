# Security Audit — kembec/gcal-mcp

**Audit date:** 2026-05-14  
**Auditor:** Claude Sonnet 4.6 (automated) — reviewed by Manuel Benancio  
**Upstream:** nspady/google-calendar-mcp

## Findings resolved in this fork

### HIGH — 7 CVEs in dependencies
- **Fixed:** `npm audit fix` applied. Runtime bundle confirmed clean (`npm audit --omit=dev`).
- Affected packages: hono, @hono/node-server, path-to-regexp, fast-uri, express-rate-limit, vite, picomatch
- Note: hono/vite/picomatch are devDependencies (not in runtime bundle)

### HIGH — CSRF in HTTP transport OAuth callback
- **Fixed:** `src/transports/http.ts` — `/oauth2callback` now validates `state` parameter, mirroring `src/auth/server.ts`
- **Original issue:** Handler accepted any `code` param without verifying the OAuth state, allowing token injection

## Findings not applicable to this deployment
- HTTP transport binding (LOW): We use stdio transport in OpenClaw, not HTTP
- CALDAV_LOG_HTTP: N/A (Google Calendar, not CalDAV)

## Credential handling (PASS — no changes needed)
- Tokens stored at `~/.config/google-calendar-mcp/tokens.json` with 0o600 permissions
- No credentials in logs or stdout
- All outbound calls to googleapis.com and accounts.google.com only
