// Fork-authored session ID handling, kept out of `client.rs` so that
// `client.rs` stays line-parallel to upstream's and merges cleanly. This is a
// child module of `client`, so it can reach the private session state that
// upstream's `AptabaseClient` keeps to itself.

use super::AptabaseClient;

/// Generates a new session ID.
pub fn new_session_id() -> String {
    super::new_session_id()
}

impl AptabaseClient {
    /// Replaces the generated session ID with a caller-supplied one, before the
    /// client is shared. An absent or empty ID leaves the generated one in
    /// place. Timeout rotation is unaffected: the SDK does not persist session
    /// IDs, so a supplied ID starts its four-hour window at construction.
    pub(crate) fn seed_session_id(&self, id: Option<String>) {
        let Some(id) = id.filter(|id| !id.is_empty()) else {
            return;
        };

        let mut session = self.session.lock().expect("could not lock events");
        session.id = id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Builder;

    #[test]
    fn new_session_id_is_not_empty() {
        // Act
        let session_id = new_session_id();

        // Assert
        assert!(!session_id.is_empty());
    }

    #[test]
    fn builder_uses_supplied_session_id() {
        // Act
        let client = Builder::new("A-DEV-123", "test")
            .with_session_id("persisted-session")
            .build();

        // Assert
        assert_eq!(client.eval_session_id(), "persisted-session");
    }

    #[test]
    fn builder_generates_session_id_when_none_supplied() {
        // Act
        let client = Builder::new("A-DEV-123", "test").build();

        // Assert
        assert!(!client.eval_session_id().is_empty());
    }

    #[test]
    fn builder_treats_empty_session_id_as_absent() {
        // Act
        let client = Builder::new("A-DEV-123", "test")
            .with_session_id("")
            .build();

        // Assert
        assert!(!client.eval_session_id().is_empty());
    }
}
