use crate::names::register_name;
use crate::server::AdapterError;
use crate::server::DapServer;
use crate::server::REGISTERS_VARIABLE_REF;
use dap::types::Variable;

impl DapServer {
    /// Handles a `variables` request. Returns the list of [Variable]s for the current execution state.
    pub(crate) fn handle_variables(&self) -> Result<Vec<Variable>, AdapterError> {
        let executor = self
            .state
            .executors
            .first()
            .ok_or(AdapterError::NoActiveTestExecutor)?;

        let variables = executor
            .interpreter
            .registers()
            .iter()
            .enumerate()
            .map(|(index, value)| Variable {
                name: register_name(index),
                value: format!("{:<8}", value),
                type_field: None,
                presentation_hint: None,
                evaluate_name: None,
                variables_reference: REGISTERS_VARIABLE_REF,
                named_variables: None,
                indexed_variables: None,
                memory_reference: None,
            })
            .collect();

        Ok(variables)
    }
}
