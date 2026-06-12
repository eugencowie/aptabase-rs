## Why

The existing Tauri plugin already contains most of a generic Aptabase Rust client, but its package and public API are coupled to Tauri. Rewriting this source tree in place as a generic crate will let Rust CLI applications reuse the proven session, batching, retry, and ingestion behavior without maintaining any Tauri integration here.

## What Changes

- **BREAKING**: Replace the root `tauri-plugin-aptabase` package in this repository with the generic `aptabase-rs` Rust client crate.
- Reuse the existing configuration, client, dispatcher, session, system metadata, batching, and retry implementations with only the changes required to remove Tauri coupling.
- Expose a public client API for configuration, event tracking, asynchronous flushing, optional panic hooks, and detached periodic flushing.
- Keep a separate `reqwest::blocking` transport out of scope; callers use the Tokio-backed flush path.
- Replace WebView-specific metadata collection with generic Rust application metadata and omit WebView engine fields from event payloads.
- Delete all Tauri-specific Rust integration, dependencies, build machinery, permissions, JavaScript bindings, generated plugin files, and Tauri examples from this repository.
- Rewrite the package metadata and documentation for the generic crate and CLI-oriented usage.

## Capabilities

### New Capabilities

- `generic-rust-client`: Configure Aptabase, enqueue typed events, manage sessions, batch and retry ingestion requests, optionally install panic hooks, and flush analytics from non-Tauri Rust applications.

### Modified Capabilities

None.

## Impact

- The repository remains a single root Rust package, renamed to `aptabase-rs`, rather than a Tauri plugin.
- The implementation in `src/config.rs`, `src/client.rs`, `src/dispatcher.rs`, and most of `src/sys.rs` will be retained and adapted in place.
- The Tauri-specific portions of `src/lib.rs`, all of `src/commands.rs`, `build.rs`, `permissions/`, `webview-src/`, `webview-dist/`, the old Tauri example, and JavaScript package metadata will be removed. A minimal Tokio CLI example remains under `examples/helloworld/`.
- Tauri and `tauri-plugin` dependencies will be removed; Tokio, Reqwest, Serde, `time`, `rand`, `os_info`, `sys-locale`, and `log` will remain where required.
- Existing users of `tauri-plugin-aptabase` cannot upgrade to this rewritten package as a compatible Tauri plugin; retaining or relocating that plugin is outside this change.
- Public lifecycle behavior must support short-lived CLI processes without requiring a Tauri application runtime.
