# Security Policy

## Security Architecture

Xmemos adheres to the **Tauri v2 Brownfield Pattern** with least-privilege capabilities:

- **No Global Tauri Injection:** `withGlobalTauri` is strictly `false`; IPC is imported exclusively via `@tauri-apps/api/core`.
- **Strict Content Security Policy (CSP):** Scripts are locked to `'self'`; media is streamed through the permission-checked `xmemos-media://localhost/<uid>` Rust protocol handler which validates SQLite memo visibility before returning bytes.
- **SQL Injection Prevention:** All queries use strictly parameterized `rusqlite::params!` bindings.
- **Path Traversal & ZIP-Slip Defense:** All `resource.reference` paths and `.xmemos`/`.zip` archive entries are canonicalized and verified to stay within `assets/` or the staging directory; `..`, absolute paths, symlink escapes, and decompression bombs (>2 GiB) are rejected.
- **XSS Sanitization:** All Markdown/KaTeX HTML is sanitized via `DOMPurify` before rendering.
- **Backend RBAC Enforcement:** Ownership, `HOST`/`ADMIN`/`USER` roles, and visibility checks are enforced inside Rust commands, never trusted from the WebView.

## Reporting a Vulnerability

Please report security issues responsibly via GitHub Security Advisories or by contacting the maintainers directly. Include steps to reproduce and affected versions. We aim to acknowledge reports within 48 hours.
