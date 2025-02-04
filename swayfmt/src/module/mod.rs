use crate::{
    comments::write_comments,
    formatter::*,
    utils::map::byte_span::{self, ByteSpan, LeafSpans},
};
use std::fmt::Write;
use sway_ast::{
    keywords::{
        ContractToken, Keyword, LibraryToken, PredicateToken, ScriptToken, SemicolonToken, Token,
    },
    Item, ItemKind, Module, ModuleKind,
};
use sway_types::Spanned;

pub(crate) mod item;
pub(crate) mod submodule;

impl Format for Module {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write_comments(formatted_code, 0..self.span().start(), formatter)?;
        self.kind.format(formatted_code, formatter)?;
        writeln!(formatted_code, "{}", SemicolonToken::AS_STR)?;

        // Format comments between module kind declaration and rest of items
        if !self.items.is_empty() {
            write_comments(
                formatted_code,
                0..self.items.first().unwrap().span().start(),
                formatter,
            )?;
        }

        let iter = self.items.iter();
        let mut prev_item: Option<&Item> = None;
        for item in iter.clone() {
            if let Some(prev_item) = prev_item {
                write_comments(
                    formatted_code,
                    prev_item.span().end()..item.span().start(),
                    formatter,
                )?;
            }

            item.format(formatted_code, formatter)?;
            if let ItemKind::Submodule { .. } = item.value {
                // Do not print a newline after a submodule
            } else {
                writeln!(formatted_code)?;
            }

            prev_item = Some(item);
        }

        if let Some(prev_item) = prev_item {
            write_comments(
                formatted_code,
                prev_item.span().end()..self.span().end(),
                formatter,
            )?;
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
            ModuleKind::Script { script_token: _ } => {
                write!(formatted_code, "{}", ScriptToken::AS_STR)?
            }
            ModuleKind::Contract { contract_token: _ } => {
                write!(formatted_code, "{}", ContractToken::AS_STR)?
            }
            ModuleKind::Predicate { predicate_token: _ } => {
                write!(formatted_code, "{}", PredicateToken::AS_STR)?
            }
            ModuleKind::Library { library_token: _ } => {
                write!(formatted_code, "{}", LibraryToken::AS_STR)?;
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
            ModuleKind::Library { library_token } => {
                vec![ByteSpan::from(library_token.span())]
            }
        }
    }
}
