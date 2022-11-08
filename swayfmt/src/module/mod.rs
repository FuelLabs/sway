use crate::{
    formatter::*,
    utils::map::byte_span::{self, ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{Module, ModuleKind};
use sway_types::Spanned;

pub(crate) mod dependency;
pub(crate) mod item;

impl Format for Module {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.kind.format(formatted_code, formatter)?;
        writeln!(formatted_code, "{}", self.semicolon_token.span().as_str())?;

        let iter = self.items.iter();
        for item in iter.clone() {
            item.format(formatted_code, formatter)?;
            writeln!(formatted_code)?;
        }

        Ok(())
    }
}

impl Format for ModuleKind {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
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
            } => {
                write!(formatted_code, "{} ", library_token.span().as_str())?;
                name.format(formatted_code, _formatter)?;
            }
        };

        Ok(())
    }
}

impl LeafSpans for Module {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![byte_span::STARTING_BYTE_SPAN];
        collected_spans.append(&mut self.kind.leaf_spans());
        collected_spans.push(ByteSpan::from(self.semicolon_token.span()));
        collected_spans.append(&mut self.items.leaf_spans());
        collected_spans
    }
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
