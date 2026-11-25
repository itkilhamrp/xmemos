# SQLite Schema Contract (AI Agent Guardrail)

## Decision Tree: Where Does New Data Belong?

1. **Does this field or table exist in Official FOSS Memos (`memos_prod.db`)?**
   - **YES:** Store it in `memos_prod.db` using the exact official column name, type, and JSON payload convention (`src-tauri/src/store/db/memos_schema.rs`).
   - **NO (Xmemos mobile/device-only feature):** Store it **EXCLUSIVELY** in `xmemos_local.db` (`src-tauri/src/store/db/local_schema.rs`).

## Immutable Guardrails
- `assert_zero_proprietary_tables` MUST pass on `memos_prod.db` in `cargo test`.
- `xmemos_local.db` is NEVER exported into `.db` or `.xmemos` archives and MUST be rebuildable from `memos_prod.db` after import.
- All `memo.content` mutations MUST recompute `memo.payload` (`tags`, `property`) and update `memo_fts` in `xmemos_local.db`.
