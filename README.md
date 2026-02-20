# Tauri + SvelteKit + TypeScript

This template should help get you started developing with Tauri, SvelteKit and TypeScript in Vite.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Optional DuckDB backend

DuckDB is behind a Cargo feature so it only compiles when explicitly enabled.

- Run without DuckDB (default):
	- `cargo tauri dev`
- Run with DuckDB (extra compile step):
	- `cargo tauri dev --features duckdb`

When enabled, the app creates a persistent database file at the Tauri app local data directory as `fidashy.duckdb`.
