# generic-rust-client Specification

## Purpose
Define the framework-independent Rust SDK surface created by extracting the existing `tauri-plugin-aptabase` client behavior for use by generic Rust applications. This covers construction, event enqueueing, baseline session lifecycle, metadata, hosted or self-hosted endpoint selection, and Tokio-based batching and flushing.

## Requirements
### Requirement: In-place generic crate replacement
The repository's root Rust package SHALL be named `aptabase-rs` and SHALL provide the generic Aptabase client. The source tree MUST NOT retain a Tauri plugin package, Tauri adapter, compatibility shim, workspace member for the old plugin, or Tauri-specific runtime and build dependencies.

#### Scenario: Build the rewritten root package
- **WHEN** a developer builds the package at the repository root
- **THEN** Cargo builds the `aptabase-rs` package directly without building or linking Tauri

#### Scenario: Inspect removed integration assets
- **WHEN** a developer inspects the rewritten source tree and package manifests
- **THEN** Tauri commands, builders, permissions, webview bindings, plugin build scripts, JavaScript package metadata, and Tauri examples are absent

### Requirement: Framework-independent client construction
The system SHALL provide a Rust Aptabase client that can be compiled and constructed without depending on Tauri or `tauri-plugin`. Construction SHALL accept an Aptabase app key, an application version, and optional client configuration.

#### Scenario: Construct client in a CLI application
- **WHEN** a Rust application constructs the client with an app key and application version
- **THEN** the client is available for event tracking and flushing without requiring an application framework or WebView

### Requirement: App-key endpoint configuration
The client SHALL select the Aptabase ingestion endpoint from the app-key region, supporting US, EU, development, and self-hosted keys. A self-hosted key MUST have a configured host. Invalid or incomplete configuration SHALL disable tracking without preventing client construction.

#### Scenario: Configure a hosted region
- **WHEN** the client is constructed with an `A-US-*` or `A-EU-*` app key
- **THEN** requests target the corresponding hosted region's `/api/v0/events` endpoint

#### Scenario: Configure a self-hosted region
- **WHEN** the client is constructed with an `A-SH-*` app key and a custom host
- **THEN** requests target the custom host's `/api/v0/events` endpoint

#### Scenario: Disable invalid configuration
- **WHEN** the client is constructed with an invalid app key or a self-hosted key without a host
- **THEN** construction succeeds and subsequent event tracking performs no network delivery

### Requirement: Event enqueueing and payload compatibility
The client SHALL synchronously enqueue named events without performing network I/O at the call site. Each event SHALL contain an RFC 3339 UTC timestamp, session ID, event name, `systemProps` with debug status, operating-system name, operating-system version, locale, application version, SDK version, and optional properties. Supplied properties MUST be a JSON object.

#### Scenario: Enqueue an event without properties
- **WHEN** a caller tracks an event name without properties on an enabled client
- **THEN** one event with the required Aptabase payload fields is added to the in-memory queue

#### Scenario: Enqueue an event with object properties
- **WHEN** a caller tracks an event with a JSON object
- **THEN** the object is included in the event's `props` field

#### Scenario: Reject non-object properties
- **WHEN** a caller tracks an event with a JSON value that is not an object
- **THEN** tracking returns a validation error and no event is enqueued

### Requirement: Session lifecycle
The client SHALL assign a session ID to tracked events, reuse that ID while activity remains within the four-hour session timeout, and generate a new ID after the timeout is exceeded.

#### Scenario: Reuse an active session
- **WHEN** multiple events are tracked less than four hours apart
- **THEN** the events contain the same session ID and the session activity time is refreshed

#### Scenario: Rotate an expired session
- **WHEN** an event is tracked more than four hours after the previous session activity
- **THEN** the event contains a newly generated session ID

### Requirement: Batched ingestion
The client SHALL flush queued events to Aptabase as JSON arrays containing no more than 25 events per request. Each request SHALL include the `App-Key` header, JSON content type, and a user agent derived from client system metadata.

#### Scenario: Flush fewer than one batch
- **WHEN** an enabled client flushes between one and 25 queued events
- **THEN** it sends one request containing all queued events to the configured ingestion endpoint

#### Scenario: Flush multiple batches
- **WHEN** an enabled client flushes more than 25 queued events
- **THEN** it sends multiple requests and no request contains more than 25 events

#### Scenario: Flush an empty queue
- **WHEN** a client flushes with no queued events
- **THEN** no network request is made

### Requirement: Delivery failure handling
The client SHALL requeue a batch after a transport failure or server-error response. The client SHALL discard a batch after a non-server HTTP error response, preserving the existing SDK delivery policy.

#### Scenario: Retry a transport failure later
- **WHEN** a flush cannot send a batch because of a network or transport error
- **THEN** the batch remains queued for a later flush

#### Scenario: Retry a server failure later
- **WHEN** Aptabase responds to a batch with a 5xx status
- **THEN** the batch remains queued for a later flush

#### Scenario: Discard a rejected request
- **WHEN** Aptabase responds to a batch with a non-5xx unsuccessful status
- **THEN** the batch is removed from the queue and is not retried automatically

### Requirement: Explicit flushing
The client SHALL provide an asynchronous flush operation that can be awaited before a short-lived process exits. The operation SHALL attempt delivery of queued batches using Tokio. The crate SHALL NOT expose a blocking flush API or provide a separate `reqwest::blocking` transport as part of this change.

#### Scenario: Flush before CLI exit
- **WHEN** a Tokio-based CLI tracks an event and awaits the client's flush operation before returning
- **THEN** the queued event is attempted before the command exits

#### Scenario: Use the supported transport model
- **WHEN** a caller needs to deliver queued events
- **THEN** the caller uses the Tokio-based asynchronous flush API

### Requirement: Optional periodic flushing
The client SHALL allow a caller with an active Tokio runtime to start detached periodic flushing using the configured interval. The periodic worker SHALL run until the Tokio runtime shuts down.

#### Scenario: Start periodic delivery
- **WHEN** a long-running application starts periodic flushing
- **THEN** the client attempts to flush queued events after each configured interval until the Tokio runtime shuts down

### Requirement: Generic system metadata
The client SHALL collect debug status, operating-system name, operating-system version, and locale without Tauri. It SHALL NOT query WebView runtime or engine metadata, and tracked events SHALL omit WebView-specific `engineName` and `engineVersion` fields.

#### Scenario: Use generic metadata
- **WHEN** a Rust application tracks an event
- **THEN** the event contains generic Rust-compatible system properties and does not query a WebView

#### Scenario: Omit WebView metadata
- **WHEN** a Rust application tracks an event
- **THEN** the event `systemProps` do not include `engineName` or `engineVersion`

### Requirement: Optional panic hooks
The builder SHALL allow callers to install either a custom panic hook or a default panic hook. When a panic hook is installed, it SHALL enqueue panic information and then delegate to Rust's previous panic hook. If the panicking thread has an active Tokio runtime, the hook SHALL also make a best-effort delivery attempt using the Tokio-backed transport before delegation. Without an active Tokio runtime on the panicking thread, the hook SHALL leave the panic event queued in memory. The panic hook MUST NOT expose a public blocking flush API.

#### Scenario: Install the default panic hook
- **WHEN** a caller enables the default panic hook
- **THEN** a panic event is enqueued with panic message and source location information before the previous panic hook runs

#### Scenario: Install a custom panic hook
- **WHEN** a caller provides a custom panic hook
- **THEN** the hook receives the client, panic information, and extracted panic message so it can enqueue an application-specific panic event
