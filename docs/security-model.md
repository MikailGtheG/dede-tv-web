# Dede TV administration: security model v0

No HTTP listener or production authentication has been implemented yet.

## Required controls
- Dedicated unprivileged service account; local API to Kodi/TV controller only.
- Bind management interface to loopback by default.
- Remote administration over Tailscale with explicit access rules; do not assume Tailnet membership alone is authorization.
- Authenticated sessions with CSRF protection for modifying actions, strict cookies and rate limits.
- Never accept unrestricted filesystem paths from the browser. Administrators select from preapproved local media roots.
- Allowlist outgoing stream protocols and reject link-local/localhost targets where URL fetch could cause SSRF.
- Do not expose Kodi JSON-RPC, SSH, database, or Tailscale control sockets to browser clients.
- Server-side audit records for configuration changes and updates; avoid sensitive token logging.

## Threats for integration review
Remote control hijacking, arbitrary media path traversal, malicious M3U/IPTV URL handling, persistent XSS via programme metadata, privilege escalation via updater, and update-signing-key compromise.
