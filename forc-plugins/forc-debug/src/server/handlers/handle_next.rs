use crate::server::AdapterError;
use crate::server::DapServer;
use forc_test::execute::DebugResult;

impl DapServer {
    /// Handles a `next` request. Returns true if the server should continue running.
    pub(crate) fn handle_next(&mut self) -> Result<bool, AdapterError> {
        self.state.update_vm_breakpoints();
        if let Some(executor) = self.state.executors.first_mut() {
            executor.interpreter.set_single_stepping(true);

            match executor.continue_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.state.test_results.push(result);
                }
                DebugResult::Breakpoint(pc) => {
                    executor.interpreter.set_single_stepping(false);
                    return self.stop_on_step(pc);
                }
            }
            self.state.executors.remove(0);
        }

        // All tests have finished
        if self.state.executors.is_empty() {
            self.log_test_results();
        }
        Ok(false)
    }
}
