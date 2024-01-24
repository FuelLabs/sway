use crate::server::AdapterError;
use crate::server::DapServer;
use dap::requests::SetBreakpointsArguments;
use dap::types::{Breakpoint, StartDebuggingRequestKind};
use std::path::PathBuf;

impl DapServer {
    /// Handles a `set_breakpoints` request. Returns the list of [Breakpoint]s for the path provided in `args`.
    pub(crate) fn handle_set_breakpoints(
        &mut self,
        args: &SetBreakpointsArguments,
    ) -> Result<Vec<Breakpoint>, AdapterError> {
        // Build the source maps so we can verify breakpoints
        if let Some(StartDebuggingRequestKind::Launch) = self.state.mode {
            let _ = self.build_tests()?;
        }

        let source_path = args
            .source
            .path
            .as_ref()
            .ok_or(AdapterError::MissingBreakpointLocation)?;

        let source_path_buf = PathBuf::from(source_path);

        let existing_breakpoints = self
            .state
            .breakpoints
            .get(&source_path_buf)
            .cloned()
            .unwrap_or_default();

        let source_map = self
            .state
            .source_map
            .get(&source_path_buf)
            .cloned()
            .unwrap_or_default();

        let breakpoints = args
            .breakpoints
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|source_bp| {
                let verified = source_map.get(&source_bp.line).is_some();

                match existing_breakpoints.iter().find(|bp| match bp.line {
                    Some(line) => line == source_bp.line,
                    None => false,
                }) {
                    Some(existing_bp) => Breakpoint {
                        verified,
                        ..existing_bp.clone()
                    },
                    None => {
                        let id = Some(self.breakpoint_id_gen.next());
                        Breakpoint {
                            id,
                            verified,
                            line: Some(source_bp.line),
                            source: Some(args.source.clone()),
                            ..Default::default()
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        self.state
            .breakpoints
            .insert(source_path_buf, breakpoints.clone());

        Ok(breakpoints)
    }
}
