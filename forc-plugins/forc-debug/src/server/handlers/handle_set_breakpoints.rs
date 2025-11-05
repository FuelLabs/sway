use crate::server::{AdapterError, DapServer, HandlerResult};
use dap::{
    requests::SetBreakpointsArguments,
    responses::ResponseBody,
    types::{Breakpoint, StartDebuggingRequestKind},
};
use std::path::PathBuf;

impl DapServer {
    /// Handles a `set_breakpoints` request. Returns the list of [Breakpoint]s for the path provided in `args`.
    pub(crate) fn handle_set_breakpoints_command(
        &mut self,
        args: &SetBreakpointsArguments,
    ) -> HandlerResult {
        let result = self.set_breakpoints(args).map(|breakpoints| {
            ResponseBody::SetBreakpoints(dap::responses::SetBreakpointsResponse { breakpoints })
        });
        match result {
            Ok(result) => HandlerResult::ok(result),
            Err(e) => HandlerResult::err_with_exit(e, 1),
        }
    }

    fn set_breakpoints(
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
            .ok_or(AdapterError::MissingSourcePathArgument)?;

        let source_path_buf = PathBuf::from(source_path);

        let existing_breakpoints = self
            .state
            .breakpoints
            .get(&source_path_buf)
            .cloned()
            .unwrap_or_default();

        let breakpoints = args
            .breakpoints
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|source_bp| {
                // Check if there are any instructions mapped to this line in the source
                let verified = self.state.source_map.map.iter().any(|(pc, _)| {
                    if let Some((path, range)) = self.state.source_map.addr_to_span(*pc) {
                        path == source_path_buf && range.start.line as i64 == source_bp.line
                    } else {
                        false
                    }
                });
                if let Some(existing_bp) = existing_breakpoints
                    .iter()
                    .find(|bp| bp.line == Some(source_bp.line))
                {
                    Breakpoint {
                        verified,
                        ..existing_bp.clone()
                    }
                } else {
                    let id = Some(self.breakpoint_id_gen.next());
                    Breakpoint {
                        id,
                        verified,
                        line: Some(source_bp.line),
                        source: Some(args.source.clone()),
                        ..Default::default()
                    }
                }
            })
            .collect::<Vec<_>>();

        self.state
            .breakpoints
            .insert(source_path_buf, breakpoints.clone());
        self.state.breakpoints_need_update = true;

        Ok(breakpoints)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sway_core::source_map::{LocationRange, PathIndex, SourceMap, SourceMapSpan};
    use sway_types::LineCol;

    const MOCK_SOURCE_PATH: &str = "some/path";
    const MOCK_BP_ID: i64 = 1;
    const MOCK_LINE: i64 = 1;
    const MOCK_INSTRUCTION: u64 = 1;

    fn get_test_server(source_map: bool, existing_bp: bool) -> DapServer {
        let mut server = DapServer::default();
        if source_map {
            // Create a source map with our test line
            let mut map = SourceMap::new();
            map.paths.push(PathBuf::from(MOCK_SOURCE_PATH));
            map.map.insert(
                MOCK_INSTRUCTION as usize,
                SourceMapSpan {
                    path: PathIndex(0),
                    range: LocationRange {
                        start: LineCol {
                            line: MOCK_LINE as usize,
                            col: 0,
                        },
                        end: LineCol {
                            line: MOCK_LINE as usize,
                            col: 10,
                        },
                    },
                },
            );
            server.state.source_map = map;
        }
        if existing_bp {
            server.state.breakpoints.insert(
                PathBuf::from(MOCK_SOURCE_PATH),
                vec![dap::types::Breakpoint {
                    id: Some(MOCK_BP_ID),
                    line: Some(MOCK_LINE),
                    verified: false,
                    source: Some(dap::types::Source {
                        path: Some(MOCK_SOURCE_PATH.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
            );
        }
        server
    }

    fn get_test_args() -> SetBreakpointsArguments {
        SetBreakpointsArguments {
            source: dap::types::Source {
                path: Some(MOCK_SOURCE_PATH.into()),
                ..Default::default()
            },
            breakpoints: Some(vec![dap::types::SourceBreakpoint {
                line: MOCK_LINE,
                ..Default::default()
            }]),
            ..Default::default()
        }
    }

    #[test]
    fn test_handle_set_breakpoints_existing_verified() {
        let mut server = get_test_server(true, true);
        let args = get_test_args();
        let result = server.set_breakpoints(&args).expect("success");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(MOCK_LINE));
        assert_eq!(result[0].id, Some(MOCK_BP_ID));
        assert_eq!(
            result[0].source.clone().expect("source").path,
            Some(MOCK_SOURCE_PATH.into())
        );
        assert!(result[0].verified);
    }

    #[test]
    fn test_handle_set_breakpoints_existing_unverified() {
        let mut server = get_test_server(false, true);
        let args = get_test_args();
        let result = server.set_breakpoints(&args).expect("success");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(MOCK_LINE));
        assert_eq!(result[0].id, Some(MOCK_BP_ID));
        assert_eq!(
            result[0].source.clone().expect("source").path,
            Some(MOCK_SOURCE_PATH.into())
        );
        assert!(!result[0].verified);
    }

    #[test]
    fn test_handle_set_breakpoints_new() {
        let mut server = get_test_server(true, false);
        let args = get_test_args();
        let result = server.set_breakpoints(&args).expect("success");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line, Some(MOCK_LINE));
        assert_eq!(
            result[0].source.clone().expect("source").path,
            Some(MOCK_SOURCE_PATH.into())
        );
        assert!(result[0].verified);
    }

    #[test]
    #[should_panic(expected = "MissingSourcePathArgument")]
    fn test_handle_breakpoint_locations_missing_argument() {
        let mut server = get_test_server(true, true);
        let args = SetBreakpointsArguments::default();
        server.set_breakpoints(&args).unwrap();
    }
}
