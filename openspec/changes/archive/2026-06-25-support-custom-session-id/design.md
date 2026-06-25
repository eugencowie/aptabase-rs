## Context

`aptabase-rs` currently creates a `TrackingSession` when `Builder::build()` constructs an `AptabaseClient`. That works for long-running processes, but short-lived applications get a new session ID on every run. `aptabase-cpp` handles this by letting the caller generate or provide a session ID and persist it outside the SDK.

## Goals / Non-Goals

**Goals:**
- Let applications reuse a persisted session ID across process runs.
- Keep the existing automatic session generation and four-hour timeout behavior.
- Match the C++ SDK's caller-owned session model with minimal Rust API surface.

**Non-Goals:**
- Do not add SDK-owned disk persistence.
- Do not add `StartSession` / `EndSession` lifecycle methods.
- Do not add a storage abstraction or new dependency.

## Decisions

- Expose the existing session ID generator as public API, likely as `aptabase_rs::new_session_id()`.
  - Alternative: force callers to invent IDs. Rejected because the SDK already has the Aptabase-compatible generator.
- Add `Builder::with_session_id(...)` to seed the initial session.
  - Alternative: add `AptabaseClient::set_session_id(...)`. Rejected because the immediate need is process startup reuse, not mutable session management.
- Treat an empty supplied session ID as absent and generate one.
  - Alternative: return a validation error. Rejected because builder methods currently do not return `Result`, and this mirrors the C++ SDK's empty-string behavior.
- Keep timeout rotation unchanged after initialization.
  - Alternative: make caller-supplied IDs permanent. Rejected because it would bypass the existing session lifecycle.

## Risks / Trade-offs

- Caller forgets to persist the generated ID -> The SDK cannot group sessions across runs; README should show caller-owned persistence explicitly.
- Caller supplies unstable or shared IDs -> Events may be grouped incorrectly; document that the application owns ID scope.
- No getter for current session ID -> Callers that want persistence should generate or load the ID before building the client.
