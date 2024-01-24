use crate::server::AdapterError;
use crate::server::DapServer;
use forc_test::execute::DebugResult;

impl DapServer {
    /// Handles a `continue` request. Returns true if the server should continue running.
    pub(crate) fn handle_continue(&mut self) -> Result<bool, AdapterError> {
        self.state.update_vm_breakpoints();

        if let Some(executor) = self.state.executors.get_mut(0) {
            executor.interpreter.set_single_stepping(false);
            match executor.continue_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.state.test_results.push(result);
                }

                DebugResult::Breakpoint(pc) => {
                    return self.stop_on_breakpoint(pc);
                }
            }
            self.state.executors.remove(0);
        }

        // If there are tests remaning, we should start debugging those until another breakpoint is hit.
        while let Some(next_test_executor) = self.state.executors.get_mut(0) {
            match next_test_executor.start_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.state.test_results.push(result);
                }
                DebugResult::Breakpoint(pc) => {
                    return self.stop_on_breakpoint(pc);
                }
            };
            self.state.executors.remove(0);
        }

        self.log_test_results();
        Ok(false)
    }
}
