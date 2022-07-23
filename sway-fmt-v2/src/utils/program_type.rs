use std::fmt::Write;
use sway_parse::{token::PunctKind, ModuleKind};
use sway_types::Spanned;

use crate::FormatterError;

/// Insert the program type without applying a formatting to it.
///
/// Possible list of program types:
///     - Script
///     - Contract
///     - Predicate
///     - Library
pub(crate) fn insert_program_type(
    formatted_code: &mut String,
    module_kind: ModuleKind,
) -> Result<(), FormatterError> {
    match module_kind {
        ModuleKind::Script { script_token } => {
            write!(formatted_code, "{}", script_token.span().as_str())?
        }

        ModuleKind::Contract { contract_token } => {
            write!(formatted_code, "{}", contract_token.span().as_str())?
        }
        ModuleKind::Predicate { predicate_token } => {
            write!(formatted_code, "{}", predicate_token.span().as_str())?
        }
        ModuleKind::Library {
            library_token,
            name,
        } => write!(
            formatted_code,
            "{} {}",
            library_token.span().as_str(),
            name.as_str()
        )?,
    };
    writeln!(formatted_code, "{}\n", PunctKind::Semicolon.as_char())?;

    Ok(())
}
