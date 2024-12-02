use crate::server::{AdapterError, DapServer};

impl DapServer {
    /// Handles a `continue` request. Returns true if the server should continue running.
    pub(crate) fn handle_continue(&mut self) -> Result<bool, AdapterError> {
        self.continue_debugging_tests(false)
    }
}
