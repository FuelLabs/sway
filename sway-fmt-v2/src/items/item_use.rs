use std::vec;

use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::comments::{CommentSpan, CommentVisitor},
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
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        if let Some(visibility) = &self.visibility {
            collected_spans.push(CommentSpan::from_span(visibility.span()));
        }
        collected_spans.push(CommentSpan::from_span(self.use_token.span()));
        if let Some(root_import) = &self.root_import {
            collected_spans.push(CommentSpan::from_span(root_import.span()));
        }
        collected_spans.append(&mut self.tree.collect_spans());
        collected_spans.push(CommentSpan::from_span(self.semicolon_token.span()));
        collected_spans
    }
}

impl CommentVisitor for UseTree {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        match self {
            UseTree::Group { imports } => imports.collect_spans(),
            UseTree::Name { name } => vec![CommentSpan::from_span(name.span())],
            UseTree::Rename {
                name,
                as_token,
                alias,
            } => vec![
                CommentSpan::from_span(name.span()),
                CommentSpan::from_span(as_token.span()),
                CommentSpan::from_span(alias.span()),
            ],
            UseTree::Glob { star_token } => vec![CommentSpan::from_span(star_token.span())],
            UseTree::Path {
                prefix,
                double_colon_token,
                suffix,
            } => {
                let mut collected_spans = vec![CommentSpan::from_span(prefix.span())];
                collected_spans.push(CommentSpan::from_span(double_colon_token.span()));
                collected_spans.append(&mut suffix.collect_spans());
                collected_spans
            }
        }
    }
}
