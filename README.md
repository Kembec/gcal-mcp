# gcal-mcp

Fork de [nspady/google-calendar-mcp](https://github.com/nspady/google-calendar-mcp) (`@cocal/google-calendar-mcp`) con parches de seguridad y compatibilidad OpenClaw. El diseño, implementación y flujo OAuth multi-cuenta son del autor original — todo el crédito es suyo.

Este fork agrega: 7 CVEs resueltos (`npm audit fix`), CSRF fix en el transporte HTTP, y bloque de configuración listo para OpenClaw. Ver [SECURITY_AUDIT.md](./SECURITY_AUDIT.md) y [OPENCLAW.md](./OPENCLAW.md).

## Qué hace

Expone tu Google Calendar como herramientas MCP: leer eventos, crear, actualizar, borrar, responder invitaciones, consultar disponibilidad. Soporta múltiples cuentas en paralelo — útil si manejás trabajo y personal por separado.

## Instalación

Necesitás un archivo de credenciales OAuth 2.0 (tipo "Desktop App") de Google Cloud Console con la Calendar API habilitada.

```bash
npx @kembec/gcal-mcp
```

Con Claude Desktop:

```json
{
  "mcpServers": {
    "google-calendar": {
      "command": "npx",
      "args": ["@kembec/gcal-mcp"],
      "env": {
        "GOOGLE_OAUTH_CREDENTIALS": "/ruta/a/gcp-oauth.keys.json"
      }
    }
  }
}
```

## Primera vez

Al iniciar, el servidor abre un browser para el flujo OAuth. Una vez autorizado, el token se guarda localmente — no necesitás repetirlo. Para agregar una segunda cuenta: pedile a Claude que use la herramienta `manage-accounts`.

Si el token expira (modo test de Google, 7 días), reautorizá con:

```bash
npx @kembec/gcal-mcp auth
```

## Herramientas disponibles

`list-calendars`, `list-events`, `search-events`, `get-event`, `create-event`, `update-event`, `delete-event`, `respond-to-event`, `get-freebusy`, `get-current-time`, `manage-accounts`

Para OpenClaw ver [OPENCLAW.md](./OPENCLAW.md).

## Créditos

Proyecto original: [nspady/google-calendar-mcp](https://github.com/nspady/google-calendar-mcp) por [@nspady](https://github.com/nspady). Publicado en npm como [`@cocal/google-calendar-mcp`](https://www.npmjs.com/package/@cocal/google-calendar-mcp).

## Licencia

MIT
