## 1. API and Session Initialization

- [x] 1.1 Expose the existing session ID generator as public `new_session_id()` API.
- [x] 1.2 Add `Builder::with_session_id(...)` and store the optional session ID on the builder.
- [x] 1.3 Pass the optional session ID into `AptabaseClient::new(...)`.
- [x] 1.4 Initialize `TrackingSession` with the non-empty supplied ID or generate one when absent/empty.

## 2. Verification and Docs

- [x] 2.1 Add focused tests for generated IDs, supplied IDs, and empty supplied IDs.
- [x] 2.2 Update README usage docs with caller-owned session ID persistence.
- [x] 2.3 Run formatting and test checks for the crate.
