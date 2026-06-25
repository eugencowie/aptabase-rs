## ADDED Requirements

### Requirement: Caller-owned session IDs
The client SHALL provide API to generate a session ID and to initialize a client with a caller-supplied session ID. When no session ID is supplied, the client SHALL continue generating one automatically. The SDK MUST NOT persist session IDs to disk.

#### Scenario: Generate a reusable session ID
- **WHEN** a caller requests a new session ID from the SDK
- **THEN** the SDK returns a non-empty session ID suitable for use as an event `sessionId`

#### Scenario: Reuse a caller-supplied session ID
- **WHEN** a client is constructed with a caller-supplied session ID and tracks an event before the session timeout
- **THEN** the event contains the supplied session ID

#### Scenario: Preserve automatic session generation
- **WHEN** a client is constructed without a caller-supplied session ID
- **THEN** tracked events contain an SDK-generated session ID

#### Scenario: Treat empty session ID as absent
- **WHEN** a client is constructed with an empty caller-supplied session ID
- **THEN** tracked events contain an SDK-generated session ID
