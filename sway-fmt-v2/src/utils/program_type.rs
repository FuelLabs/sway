use sway_parse::ModuleKind;
use sway_types::Spanned;

/// Insert the program type without applying a formatting to it.
///
/// Possible list of program types:
///     - Script
///     - Contract
///     - Predicate
///     - Library
pub(crate) fn insert_program_type(push_to: &mut String, module_kind: ModuleKind) {
    match module_kind {
        ModuleKind::Script { script_token } => push_to.push_str(script_token.span().as_str()),
        ModuleKind::Contract { contract_token } => push_to.push_str(contract_token.span().as_str()),
        ModuleKind::Predicate { predicate_token } => {
            push_to.push_str(predicate_token.span().as_str())
        }
        ModuleKind::Library {
            library_token,
            name,
        } => {
            push_to.push_str(library_token.span().as_str());
            push_to.push(' ');
            push_to.push_str(name.as_str());
        }
    };
    push_to.push(';');
    push_to.push('\n');
    push_to.push('\n');
}
