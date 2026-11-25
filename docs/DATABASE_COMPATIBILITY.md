# 1:1 Official Memos SQLite Database Compatibility

Xmemos guarantees **1:1 schema, binary, and round-trip compatibility** with Official FOSS Memos (`memos_prod.db`).

## Table Parity Matrix (`memos_prod.db`)

| Table | Official Memos Parity | Notes |
| :--- | :--- | :--- |
| `migration_history` | Exact | Tracks compatible schema version (`0.24.0`). |
| `system_setting` | Exact | Stores `disallow-user-registration` and workspace settings. |
| `user` | Exact | Roles `HOST`/`ADMIN`/`USER`; `bcrypt` (`$2a$`/`$2b$`, cost 10) password hashes. |
| `user_setting` | Exact | Per-user key/value settings. |
| `memo` | Exact | `visibility` (`PRIVATE`/`PROTECTED`/`PUBLIC`), `pinned`, structured JSON `payload` (`tags`, `property`, `location`). |
| `memo_relation` | Exact | `COMMENT` and `REFERENCE` relationships. |
| `resource` | Exact | Supports `storage_type = 'LOCAL'` (`assets/YYYY/MM/...`) and legacy `blob` attachments. |
| `activity` | Exact | Activity log preserved across round-trips. |
| `inbox` | Exact | Multi-user local comment and reaction notifications. |
| `webhook` | Preserved | Retained intact when importing/exporting from a Memos server. |
| `reaction` | Exact | Emoji reactions keyed by `memos/<uid>`. |
| `idp` | Preserved | Identity provider rows preserved untouched during round-trips. |

## Dual-Mode Backup & Migration

### Mode 1: Database-Only (`.db`)
Executes `PRAGMA wal_checkpoint(TRUNCATE)` followed by `VACUUM INTO` and `PRAGMA journal_mode = DELETE` + `PRAGMA integrity_check`. Rename the exported `.db` file to `memos_prod.db` and place it directly inside an official Memos server's `--data` directory to boot immediately.

### Mode 2: Full Bundle (`.xmemos`)
A standard ZIP archive containing `memos_prod.db`, `assets/` (matching `resource.reference`), and `manifest.json` (SHA-256 checksums for every file). Unzipping directly into `/var/opt/memos/` on a Linux server yields a complete self-hosted Memos instance with all static media intact.

### Importing From Official Memos Into Xmemos
1. Stop your official Memos server (or run `VACUUM INTO` on `memos_prod.db`).
2. Copy `memos_prod.db` (or zip `memos_prod.db` + `assets/`) to your Android device.
3. Open **Settings -> Storage & Backup -> Choose Backup File (SAF)**.
4. Review the pre-import summary (user/memo/resource counts, SHA-256 verification) and confirm **Replace & Restore**.
5. If your server stored media inside `resource.blob`, tap **Externalize** in Storage Settings to move BLOBs into `/Android/media/org.xmemos.app/assets/` while preserving `resource.uid`.

## Backup Scope & Official Memos RBAC (v2.0.9+)

- **All Users (Workspace) Export (`ALL_USERS`)**: Restricted strictly to `HOST` and `ADMIN` accounts. Exports the complete `memos_prod.db` (all local accounts, memos, comments, reactions, settings, and resources) and all files in `assets/`.
- **Active User Only Export (`ACTIVE_USER`)**: Available to all authenticated accounts (and enforced as the sole export scope for `USER` accounts). Produces a clean, standalone 1:1 Official Memos SQLite database (`memos_prod.db`) pruned to only the active user's records (with the single exported user promoted to `HOST` so Official Memos can boot directly from it) and, for `.xmemos` bundles, only the static media files referenced by that user's resources.
- **Safe Atomic Workspace Restore**: Restricted to `HOST` / `ADMIN` once a workspace has been initialized.
