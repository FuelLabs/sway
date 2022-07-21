use std::vec;

use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::comments::{ByteSpan, CommentVisitor},
    FormatterError,
};
use sway_parse::{ItemUse, UseTree};
use sway_types::Spanned;

impl Format for ItemUse {
    fn format(
        &self,
        _formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        todo!()
    }
}

impl CommentVisitor for ItemUse {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
            collected_spans.push(ByteSpan::from_span(visibility.span()));
        }
        collected_spans.push(ByteSpan::from_span(self.use_token.span()));
        if let Some(root_import) = &self.root_import {
            collected_spans.push(ByteSpan::from_span(root_import.span()));
        }
        collected_spans.append(&mut self.tree.collect_spans());
        collected_spans.push(ByteSpan::from_span(self.semicolon_token.span()));
        collected_spans
    }
}

impl CommentVisitor for UseTree {
    fn collect_spans(&self) -> Vec<ByteSpan> {
        match self {
            UseTree::Group { imports } => imports.collect_spans(),
            UseTree::Name { name } => vec![ByteSpan::from_span(name.span())],
            UseTree::Rename {
                name,
                as_token,
                alias,
            } => vec![
                ByteSpan::from_span(name.span()),
                ByteSpan::from_span(as_token.span()),
                ByteSpan::from_span(alias.span()),
            ],
            UseTree::Glob { star_token } => vec![ByteSpan::from_span(star_token.span())],
            UseTree::Path {
                prefix,
                double_colon_token,
                suffix,
            } => {
                let mut collected_spans = vec![ByteSpan::from_span(prefix.span())];
                collected_spans.push(ByteSpan::from_span(double_colon_token.span()));
                collected_spans.append(&mut suffix.collect_spans());
                collected_spans
            }
        }
    }
}
