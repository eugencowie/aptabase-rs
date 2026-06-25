## Why

Short-lived programs create a new in-memory Aptabase session on every run, so events from repeated invocations cannot be grouped unless the application can persist and reuse a session ID. The C++ SDK handles this by letting callers provide a custom session ID; `aptabase-rs` should expose the same minimal control without owning disk persistence.

## What Changes

- Add a public session ID generator so applications can create SDK-compatible session IDs when they need one.
- Add a builder option for supplying an initial session ID.
- Continue generating a session ID automatically when the caller does not provide one.
- Keep session storage and persistence outside the SDK.

## Capabilities

### New Capabilities
- `custom-session-id`: Caller-owned session ID generation and initialization for applications that persist session state across process runs.

### Modified Capabilities
- None.

## Impact

- Public Rust API: `Builder` gains a custom-session option and the crate exposes session ID generation.
- Session initialization changes in `src/client.rs`; automatic timeout rotation remains unchanged.
- README usage docs should show caller-owned persistence for short-lived applications.
- Tests should cover generated IDs and caller-supplied session IDs.
