use crate::server::AdapterError;
use crate::server::DapServer;
use forc_test::execute::DebugResult;

impl DapServer {
    /// Handle a `continue` request. Returns true if the server should continue running.
    pub(crate) fn handle_next(&mut self) -> Result<bool, AdapterError> {
        self.update_vm_breakpoints();
        if let Some(executor) = self.executors.get_mut(0) {
            executor.interpreter.set_single_stepping(true);

            match executor.continue_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.test_results.push(result);
                }

                DebugResult::Breakpoint(pc) => {
                    executor.interpreter.set_single_stepping(false);
                    return self.stop_on_step(pc);
                }
            }
            self.executors.remove(0);
        }

        // All tests have finished
        if self.executors.len() == 0 {
            self.log_test_results();
        }
        return Ok(false);
    }
}
