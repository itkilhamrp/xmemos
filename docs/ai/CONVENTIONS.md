# Coding Conventions (AI Agent Reference)

## Svelte 5 & SvelteKit 3 Rules (Strict)
- Always use `<script lang="ts">` and Svelte 5 Runes (`$state`, `$derived`, `$effect`, `$props`, `$bindable`, `{#snippet ...}`, `{@render ...}`).
- **Strictly Forbidden:** `export let`, `$:`, `createEventDispatcher`, `writable()`/`readable()` stores, `<slot>`, legacy `$lib` imports, and deprecated `sveltekit({ alias })` config.
- **SvelteKit 3 `#lib/*` Subpath Imports:** Declare `"imports": { "#lib/*": "./src/lib/*" }` in `package.json` and always include explicit module extensions (`.ts`, `.svelte.ts`, `.svelte`) on `#lib/*` imports (e.g., `import { authState } from '#lib/state/auth.svelte.ts'`). Do not override `compilerOptions.paths` in `tsconfig.json`.
- **Safe `$effect` Discipline:** Never read and write the same `$state` variable synchronously inside `$effect` (use `untrack()` when reading reactive state solely for cleanup such as `URL.revokeObjectURL`).

## TypeScript Rules (Strict)
- Zero `.js` files inside `src/`.
- `strict: true`, `noUncheckedIndexedAccess: true`, `noImplicitOverride: true`.
- Zero `any`. Define explicit DTO interfaces in `src/lib/types/` mirroring Rust `#[serde(rename_all = "camelCase")]` structs.

## Konsta UI v5 iOS + Liquid Glass & WCAG 2.2 Rules
- Root `<App theme="ios" dark={true} safeAreas>`. Never use `theme="material"`. Keep `dark={true}` so Konsta UI v5 always emits `dark:` Tailwind classes controlled by `<html class="dark">`.
- Use native Konsta UI v5 `<Navbar>`, `<NavbarBackLink>`, `<Tabbar>` + `<ToolbarPane>`, `<Sheet>`, `<Panel>`, `<BlockTitle>`, `<List>`, `<Dialog>`, and `<Segmented>` without double-wrapping them in conflicting `.glass-surface` / `.glass-pill` fixed containers.
- Inside Konsta UI v5 `<Dialog>`, place body text in the default `children` slot (not `{#snippet content()}`), and style destructive buttons with solid iOS red (`!bg-[#d70015] dark:!bg-[#ff453a] !text-white`) and concise single-line labels.
- For search bars, use `<input type="text" inputmode="search" enterkeyhint="search" role="searchbox">` instead of `type="search"` so WebKitGTK never renders a duplicate native clear button.
- Pair `<BlockTitle class="!mt-5 !mb-2 !px-1">` with `<List strong inset class="!my-0 !mx-0">` so Konsta UI's default negative `-mb-6` title margin never overlaps inset lists.
- Apply semantic high-contrast tokens from `src/app.css` (`.xm-text-primary`, `.xm-text-secondary`, `.xm-text-muted`, `.xm-text-accent`, `.xm-input`) rather than low-contrast `opacity-*` utilities.
- Preserve pinch-to-zoom (`viewport` without `user-scalable=no`), global `:focus-visible` rings, ARIA labels/roles, and 40–44px touch targets.
- Animate only `transform` and `opacity`; never animate `backdrop-filter` blur radius on scroll.

## Rust Rules
- Zero `.unwrap()` or `.expect()` inside `src-tauri/src/commands/`. Return `AppResult<T>`.
- Use strictly parameterized `rusqlite::params!` queries.
- Enforce authentication, RBAC (`HOST`/`ADMIN`/`USER`), and visibility rules in Rust before touching SQLite.
