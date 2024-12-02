use crate::server::{AdapterError, DapServer};

impl DapServer {
    /// Handles a `next` request. Returns true if the server should continue running.
    pub(crate) fn handle_next(&mut self) -> Result<bool, AdapterError> {
        self.continue_debugging_tests(true)
    }
}
