## 1. Rewrite The Root Package

- [x] 1.1 Rename the root Cargo package to `aptabase-rs` and rewrite its metadata for the generic client without introducing a workspace or secondary crate.
- [x] 1.2 Remove the `tauri` dependency, `tauri-plugin` build dependency, plugin `links` metadata, and any features or metadata that only apply to Tauri.
- [x] 1.3 Define the root crate's public module surface and exports for client construction, options, metadata overrides, event tracking, and flushing.

## 2. Adapt Configuration And Metadata

- [x] 2.1 Move `InitOptions` into the generic root API and adapt app-key configuration in place, preserving US, EU, development, self-hosted, invalid-key, and flush-interval behavior.
- [x] 2.2 Remove `tauri::webview_version` from system metadata collection while retaining debug status, OS information, locale, and Flatpak detection.
- [x] 2.3 Add generic runtime or engine defaults and public configuration overrides for engine name and version.
- [x] 2.4 Add unit tests for hosted, development, self-hosted, and invalid app-key configuration and metadata overrides.

## 3. Adapt The Client And Dispatcher

- [x] 3.1 Retain the session ID and four-hour session lifecycle implementation in the rewritten root crate.
- [x] 3.2 Adapt event construction and synchronous enqueueing, preserving payload field names, application and SDK versions, disabled-client behavior, and JSON-object property validation.
- [x] 3.3 Retain the Reqwest dispatcher behavior for the ingestion path, request headers, user agent, ten-second timeout, in-memory queue, and 25-event batching.
- [x] 3.4 Preserve requeue behavior for transport and 5xx failures and discard behavior for non-5xx unsuccessful responses.
- [x] 3.5 Expose asynchronous explicit flushing for CLI shutdown and optional periodic flushing with a caller-manageable task handle or cancellation mechanism.
- [x] 3.6 Verify the public API and dependency features do not introduce a runtime-independent blocking client or `reqwest::blocking` transport.
- [x] 3.7 Remove the old Tauri-oriented `Builder`, `PanicHook`, and `EventTracker` public APIs and replace `src/lib.rs` with the generic crate API.

## 4. Remove Tauri From The Source Tree

- [x] 4.1 Delete `src/commands.rs` and all remaining Tauri imports, command handlers, managed-state integration, plugin builders, runtime event hooks, and WebView metadata code.
- [x] 4.2 Delete the root plugin `build.rs` and the `permissions/` directory, including generated schemas and command permissions.
- [x] 4.3 Delete `webview-src/`, `webview-dist/`, `package.json`, and other JavaScript or TypeScript build metadata used only by the Tauri plugin.
- [x] 4.4 Delete `examples/helloworld/` and all generated Tauri example artifacts.
- [x] 4.5 Search the retained source, manifests, and documentation and remove all claims or configuration that this repository still provides a Tauri plugin.

## 5. Verify Core Behavior

- [x] 5.1 Add tests that verify event payload serialization, required system properties, object-property acceptance, and non-object rejection.
- [x] 5.2 Add deterministic session tests for reuse within four hours and rotation after expiration.
- [x] 5.3 Add local HTTP test coverage for empty flushes, `App-Key` and content headers, endpoint selection, and batches of at most 25 events.
- [x] 5.4 Add delivery tests showing that transport and 5xx failures remain queued while non-5xx unsuccessful responses are discarded.
- [x] 5.5 Add a Tokio-based CLI example or integration test that tracks an event and awaits `flush` using the rewritten root crate.

## 6. Documentation And Final Validation

- [x] 6.1 Replace the Tauri README with generic crate documentation covering CLI construction, event tracking, explicit shutdown flushing, periodic flushing, self-hosted configuration, and privacy guidance.
- [x] 6.2 Update changelog and package metadata to state that this is a breaking in-place replacement named `aptabase-rs`.
- [x] 6.3 Run formatting, unit and integration tests, Cargo checks, and documentation tests for the rewritten root package.
- [x] 6.4 Inspect the dependency tree and repository files to verify that no Tauri dependency, Tauri source, plugin asset, or compatibility layer remains.
