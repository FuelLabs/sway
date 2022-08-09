use crate::{
    utils::byte_span::{ByteSpan, LeafSpans},
    FormatterError,
};
use std::fmt::Write;
use sway_ast::{dependency::DependencyPath, token::PunctKind, Dependency, Module, ModuleKind};
use sway_types::Spanned;

/// Insert the program type without applying a formatting to it.
///
/// Possible list of program types:
///     - Script
///     - Contract
///     - Predicate
///     - Library
pub(crate) fn insert_program_type(
    formatted_code: &mut String,
    module_kind: &ModuleKind,
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

impl LeafSpans for ModuleKind {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            ModuleKind::Script { script_token } => {
                vec![ByteSpan::from(script_token.span())]
            }
            ModuleKind::Contract { contract_token } => {
                vec![ByteSpan::from(contract_token.span())]
            }
            ModuleKind::Predicate { predicate_token } => {
                vec![ByteSpan::from(predicate_token.span())]
            }
            ModuleKind::Library {
                library_token,
                name,
            } => {
                vec![
                    ByteSpan::from(library_token.span()),
                    ByteSpan::from(name.span()),
                ]
            }
        }
    }
}

impl LeafSpans for Module {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = self.kind.leaf_spans();
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans.append(&mut self.dependencies.leaf_spans());
        collected_spans.append(&mut self.items.leaf_spans());
        collected_spans
    }
}

impl LeafSpans for Dependency {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.dep_token.span())];
        collected_spans.append(&mut self.path.leaf_spans());
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans
    }
}

impl LeafSpans for DependencyPath {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = self.prefix.leaf_spans();
        collected_spans.append(&mut self.suffixes.leaf_spans());
        collected_spans
    }
}
