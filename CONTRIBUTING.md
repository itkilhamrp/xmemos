# Contributing to Xmemos

Thank you for contributing to Xmemos!

## Workflow & Standards

1. **Conventional Commits:** Prefix commit messages with `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`, or `ci:`.
2. **Strict TypeScript Only:** Never add `.js` files inside `src/`. All frontend components must use Svelte 5 Runes (`$state`, `$derived`, `$effect`, `$props`).
3. **Konsta UI iOS Only:** Always use `konsta/svelte` with `theme="ios"` and Liquid Glass utility classes from `src/app.css`.
4. **1:1 SQLite Compatibility:** Never add proprietary columns or tables to `memos_prod.db`. Device-only state belongs exclusively in `xmemos_local.db`.
5. **Single Source of Truth for Version:** Update `"version"` only in `package.json`, then run `pnpm version:sync`.

## Mandatory Pre-PR Verification

```bash
pnpm license:apply
pnpm license:check
pnpm version:check
pnpm check
pnpm test
pnpm build
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
```
