use crate::server::{AdapterError, DapServer, HandlerResult};
use dap::{
    requests::BreakpointLocationsArguments, responses::ResponseBody, types::BreakpointLocation,
};
use std::path::PathBuf;

impl DapServer {
    /// Handles a `breakpoint_locations` request. Returns the list of [BreakpointLocation]s.
    pub(crate) fn handle_breakpoint_locations_command(
        &self,
        args: &BreakpointLocationsArguments,
    ) -> HandlerResult {
        let result = self.breakpoint_locations(args).map(|breakpoints| {
            ResponseBody::BreakpointLocations(dap::responses::BreakpointLocationsResponse {
                breakpoints,
            })
        });
        match result {
            Ok(result) => HandlerResult::ok(result),
            Err(e) => HandlerResult::err_with_exit(e, 1),
        }
    }

    fn breakpoint_locations(
        &self,
        args: &BreakpointLocationsArguments,
    ) -> Result<Vec<BreakpointLocation>, AdapterError> {
        let source_path = args
            .source
            .path
            .as_ref()
            .ok_or(AdapterError::MissingSourcePathArgument)?;

        let existing_breakpoints = self
            .state
            .breakpoints
            .get(&PathBuf::from(source_path))
            .ok_or(AdapterError::MissingBreakpointLocation)?;

        let breakpoints = existing_breakpoints
            .iter()
            .filter_map(|bp| {
                bp.line.map(|line| BreakpointLocation {
                    line,
                    ..Default::default()
                })
            })
            .collect();

        Ok(breakpoints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOCK_SOURCE_PATH: &str = "some/path";
    const MOCK_BP_ID: i64 = 1;
    const MOCK_LINE: i64 = 1;

    #[test]
    fn test_handle_breakpoint_locations_success() {
        let mut server = DapServer::default();
        server.state.breakpoints.insert(
            PathBuf::from(MOCK_SOURCE_PATH),
            vec![dap::types::Breakpoint {
                id: Some(MOCK_BP_ID),
                line: Some(MOCK_LINE),
                ..Default::default()
            }],
        );
        let args = BreakpointLocationsArguments {
            source: dap::types::Source {
                path: Some(MOCK_SOURCE_PATH.into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = server.breakpoint_locations(&args).expect("success");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, MOCK_LINE);
    }

    #[test]
    #[should_panic(expected = "MissingSourcePathArgument")]
    fn test_handle_breakpoint_locations_missing_argument() {
        let server = DapServer::default();
        let args = BreakpointLocationsArguments::default();
        server.breakpoint_locations(&args).unwrap();
    }

    #[test]
    #[should_panic(expected = "MissingBreakpointLocation")]
    fn test_handle_breakpoint_locations_missing_breakpoint() {
        let server = DapServer::default();
        let args = BreakpointLocationsArguments {
            source: dap::types::Source {
                path: Some(MOCK_SOURCE_PATH.into()),
                ..Default::default()
            },
            ..Default::default()
        };
        server.breakpoint_locations(&args).unwrap();
    }
}
