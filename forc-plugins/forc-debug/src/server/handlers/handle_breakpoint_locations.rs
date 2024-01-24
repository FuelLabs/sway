use crate::server::AdapterError;
use crate::server::DapServer;
use dap::requests::BreakpointLocationsArguments;
use dap::types::BreakpointLocation;
use std::path::PathBuf;

impl DapServer {
    /// Handles a `breakpoint_locations` request. Returns the list of [BreakpointLocation]s.
    pub(crate) fn handle_breakpoint_locations(
        &mut self,
        args: &BreakpointLocationsArguments,
    ) -> Result<Vec<BreakpointLocation>, AdapterError> {
        let source_path = args
            .source
            .path
            .as_ref()
            .ok_or(AdapterError::MissingBreakpointLocation)?;

        let existing_breakpoints = self
            .state
            .breakpoints
            .get(&PathBuf::from(source_path))
            .ok_or(AdapterError::MissingBreakpointLocation)?;

        let breakpoints = existing_breakpoints
            .iter()
            .filter_map(|bp| {
                if let Some(line) = bp.line {
                    return Some(BreakpointLocation {
                        line,
                        ..Default::default()
                    });
                }
                None
            })
            .collect();

        Ok(breakpoints)
    }
}
