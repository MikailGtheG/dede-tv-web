# Dede TV Web

Remote control and administration for Dede TV OS. **Pre-release, not production ready.**

## Offline, interactive design prototype

Open `prototype/index.html` in a modern browser (or serve this directory
locally). It implements a TV simulation and an accessible D-pad with keyboard
support. The UI demonstrates Dede TV's dark silver, large-type design.

- **Enter/OK** opens a channel list and confirms selection.
- **Arrow Up/Down** navigates the overlay.
- **Escape/Backspace** closes it.
- **PageUp/PageDown** switch channel directly.
- The demo saves the selected channel in browser-local storage.

This is intentionally a **local mock**; it does not contact Kodi or the Rust
Core, and there is currently no network or login functionality. Do not expose
it as if it were a secure administration interface.

## Security design

See `docs/security-model.md`. Production integration requires a Rust service
with authenticated sessions, restricted Unix/loopback control, explicit
authorized media roots, CSRF protection and operational audit logs.

## Tests

The Web Static GitHub Actions job validates JavaScript syntax, unique HTML
element IDs, local-only script/styles, and a strict Content Security Policy.
These checks do not replace browser/device accessibility tests.
