# Rust SDK for Aptabase

A client that records analytics events in a Rust application and sends them to an Aptabase server.

## Language

### Core concepts

**App Key**:
The identifier Aptabase issues to an application, which names the region its events go to. An invalid app key disables tracking.
_Avoid_: API key, token

**Region**:
The Aptabase server an app key belongs to: `US` or `EU` cloud, `DEV` for a local server, or `SH` for a self-hosted server at a host the application supplies.
_Avoid_: Environment, server

**Event**:
A single named occurrence in the application, with optional custom properties, enriched by the SDK with the session, app version and system details.
_Avoid_: Message, hit

**Session**:
A run of activity that groups events, identified by a session ID. It ends after 4 hours without events, and on application restart unless the application supplies a persisted session ID.
_Avoid_: Visit

### Event delivery

**Flush**:
Sending every event not yet delivered to the server.
_Avoid_: Sync

**Batch**:
The events sent to the server together in one request.
_Avoid_: Chunk, page

**Polling**:
Flushing automatically at a fixed interval, as opt-in behaviour for long-running applications.
_Avoid_: Auto-flush, background sync

**Delivery**:
How a flush gets events to the server: how events are grouped into batches, and which failed batches are kept for the next flush and which are dropped.
_Avoid_: Dispatch, upload
