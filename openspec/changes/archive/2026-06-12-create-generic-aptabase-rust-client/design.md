## Context

The current root package combines a mostly framework-independent Aptabase client with a Tauri-specific shell. `config.rs`, `client.rs`, and `dispatcher.rs` contain the reusable behavior, while `lib.rs`, `commands.rs`, `build.rs`, permissions, webview bindings, package metadata, and examples make this repository a Tauri plugin. The only direct Tauri dependency inside the otherwise reusable modules is WebView version collection in `sys.rs`.

This change is an in-place rewrite of the existing source tree. The repository will not become a workspace and will not retain a Tauri adapter. The root package itself becomes the generic `aptabase-rs` crate, and all Tauri-specific code and assets are deleted.

## Goals / Non-Goals

**Goals:**

- Convert the repository's root package into the `aptabase-rs` crate with no Tauri dependency.
- Retain and adapt the existing configuration, session, event construction, queue, batching, retry, and HTTP dispatch code with minimal behavioral changes.
- Provide explicit asynchronous flushing suitable for short-lived CLI applications and optional periodic flushing for long-running applications.
- Preserve the Aptabase ingestion endpoint, headers, batch size, event field names, session behavior, and retry policy.
- Remove every Tauri-specific source file, dependency, build artifact definition, permission, frontend binding, example, and documentation path from this repository.
- Add focused tests around the retained behavior.

**Non-Goals:**

- Preserving the Tauri plugin API or maintaining source compatibility for existing plugin consumers.
- Keeping a Tauri adapter, compatibility crate, workspace member, or deprecated shim in this repository.
- Adding automatic command, argument, crash, or usage tracking to CLI applications.
- Persisting unsent events to disk or bounding the in-memory queue in this change.
- Changing Aptabase's event schema, ingestion protocol, or retry semantics.
- Providing a synchronous HTTP implementation independent of Tokio.
- Publishing `aptabase-rs` to crates.io as part of the code change.
- Providing a runtime-independent blocking client or a `reqwest::blocking` transport.

## Decisions

### Rewrite the root package in place

The existing root `Cargo.toml`, `src/`, README, and repository metadata will be rewritten for the `aptabase-rs` package. No Cargo workspace or secondary crate directory will be introduced.

This directly matches the intended repository ownership and minimizes structural churn. A workspace containing both a generic core and the Tauri plugin was considered and rejected because the Tauri code is no longer wanted in this repository.

### Delete Tauri integration rather than preserve an adapter

The Tauri builder, managed state, command, `EventTracker` implementations, panic-hook integration, exit lifecycle integration, plugin build script, permissions, webview bindings, JavaScript package, and Tauri example will be removed.

Retaining a compatibility adapter was considered, but it would preserve dependencies and maintenance responsibilities that this rewrite is explicitly intended to eliminate. This is therefore a breaking package replacement rather than a backward-compatible extraction.

### Adapt existing modules instead of reimplementing the protocol

The generic crate will be based directly on the existing `config`, `client`, `dispatcher`, and `sys` modules. Type and method names may be simplified at the public boundary, but event construction, session generation, batching, and retry code should remain structurally recognizable.

A ground-up implementation was considered, but it would increase the risk of changing established payload details and offline behavior for little benefit.

### Make the client own framework-independent lifecycle operations

The root crate will expose a client that supports:

- construction from an app key, app version, and options;
- synchronous event enqueueing;
- asynchronous `flush`;
- optional periodic flushing started from an active Tokio runtime.

Explicit `flush().await` is the CLI shutdown mechanism. Periodic flushing remains available for long-running applications. A runtime-independent blocking client is explicitly outside this proposal and will not be implemented.

Using only a detached background task was considered, but short-lived CLI commands can terminate before queued events are transmitted. A separate `reqwest::blocking` implementation was also considered, but it would duplicate transport code and expand the initial scope.

### Preserve tolerant initialization and event behavior

Invalid or incomplete app-key configuration will continue to disable tracking and log a diagnostic instead of preventing application startup. Event enqueueing will remain a no-op while disabled. Event properties must remain either absent or a JSON object.

Introducing a fully fallible constructor and a new public error hierarchy was considered. That would be reasonable for a new SDK, but it conflicts with the goal of minimal adaptation. Public error refinement can follow as a separate change.

### Replace WebView metadata with generic runtime metadata

The crate will collect OS name, OS version, locale, and debug status without Tauri. It will provide generic Rust runtime or engine defaults and allow callers to override the engine name and version used in `systemProps` and the user agent.

Removing the engine fields was considered, but preserving the payload shape is lower risk and keeps the retained event construction close to the existing implementation.

### Keep the current in-memory batching and retry policy

The dispatcher will continue to send at most 25 events per request, requeue events after network failures or server errors, and discard events rejected with non-server HTTP errors. The queue will remain process-local and in-memory.

Changing retry policy, adding backoff, persistence, queue limits, or delivery acknowledgements would make this an SDK redesign rather than a low-change rewrite.

### Do not install global panic hooks

The generic client will provide tracking and flushing primitives but will not install or manage a global panic hook. Panic handling is application policy, and the existing implementation belongs to the Tauri shell that is being removed.

## Risks / Trade-offs

- [The rewrite breaks all existing Tauri plugin consumers] -> Mark the package conversion as breaking and rewrite documentation so the repository no longer claims Tauri compatibility.
- [In-place deletion removes a potentially useful plugin history from the working tree] -> Rely on Git history; do not retain dead compatibility code in the new crate.
- [Adaptation accidentally changes event payloads] -> Add serialization tests using representative events and verify required field names and metadata behavior.
- [CLI processes exit before delivery] -> Document and test explicit `flush().await` as part of normal command shutdown.
- [A periodic task can outlive useful client state or be difficult to stop] -> Return a task handle or cancellation guard so callers can explicitly manage it.
- [Concurrent flushes can reorder delivery] -> Preserve current locking behavior and document that callers should start at most one periodic worker per client.
- [Unbounded in-memory retry queue can grow during long outages] -> Document the inherited limitation and defer queue limits or persistence to a follow-up change.

## Migration Plan

1. Rewrite the root package metadata and public module surface for the generic Aptabase crate.
2. Adapt the existing configuration and system metadata modules to remove Tauri references.
3. Adapt the existing client and dispatcher modules and expose the generic tracking and flushing API.
4. Add unit and local HTTP integration tests for retained behavior.
5. Delete Tauri-only Rust code, dependencies, build scripts, permissions, webview bindings, JavaScript metadata, generated files, examples, and Tauri-focused documentation.
6. Run formatting, tests, checks, documentation tests, and dependency inspection on the rewritten root package.

Rollback consists of reverting the change in Git. No persisted data or external migration is involved.

## Open Questions

None. The crates.io package name is `aptabase-rs`, and runtime-independent blocking client support is outside this proposal.
