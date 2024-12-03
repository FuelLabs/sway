use crate::server::{util, AdapterError, DapServer, HandlerResult};
use dap::{
    responses::ResponseBody,
    types::{StackFrame, StackFramePresentationhint},
};

impl DapServer {
    /// Handles a `stack_trace` request. Returns the list of [StackFrame]s for the current execution state.
    pub(crate) fn handle_stack_trace_command(&self) -> HandlerResult {
        let result = self.stack_trace().map(|stack_frames| {
            ResponseBody::StackTrace(dap::responses::StackTraceResponse {
                stack_frames,
                total_frames: None,
            })
        });
        match result {
            Ok(result) => HandlerResult::ok(result),
            Err(e) => HandlerResult::err_with_exit(e, 1),
        }
    }

    fn stack_trace(&self) -> Result<Vec<StackFrame>, AdapterError> {
        let executor = self
            .state
            .executors
            .first()
            .ok_or(AdapterError::NoActiveTestExecutor)?;
        let name = executor.name.clone();

        let source_location = match self.state.stopped_on_breakpoint_id {
            // If we stopped on a breakpoint, use the breakpoint's source location.
            Some(breakpoint_id) => self.state.breakpoints.iter().find_map(|(_, breakpoints)| {
                breakpoints.iter().find_map(|bp| {
                    if Some(breakpoint_id) == bp.id {
                        if let Some(bp_line) = bp.line {
                            return Some((bp.source.clone(), bp_line));
                        }
                    }
                    None
                })
            }),
            // Otherwise, use the current instruction's source location.
            None => self
                .state
                .vm_pc_to_source_location(util::current_instruction(
                    executor.interpreter.registers(),
                ))
                .ok()
                .map(|(source_path, line)| (Some(util::path_into_source(source_path)), line)),
        };

        // For now, we only return 1 stack frame.
        let stack_frames = source_location
            .map(|(source, line)| {
                vec![StackFrame {
                    id: 0,
                    name,
                    source,
                    line,
                    column: 0,
                    presentation_hint: Some(StackFramePresentationhint::Normal),
                    ..Default::default()
                }]
            })
            .unwrap_or_default();
        Ok(stack_frames)
    }
}
