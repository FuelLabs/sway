/// Associated metadata attached mostly to values.
///
/// Each value (instruction, function argument or constant) has associated metadata which helps
/// describe properties which aren't required for code generation, but help with other
/// introspective tools (e.g., the debugger) or compiler error messages.
///
/// NOTE: At the moment the Spans contain a source string and optional path.  Any spans with no
/// path are ignored/rejected by this module.  The source string is not (de)serialised and so the
/// string is assumed to always represent the entire contents of the file path.
use std::sync::Arc;

use sway_types::span::Span;

use crate::{context::Context, error::IrError};

pub enum Metadatum {
    FileLocation(Arc<std::path::PathBuf>, Arc<str>),
    Span {
        loc_idx: MetadataIndex,
        start: usize,
        end: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MetadataIndex(pub generational_arena::Index);

impl MetadataIndex {
    pub fn from_span(context: &mut Context, span: &Span) -> Option<MetadataIndex> {
        // Search for an existing matching path, otherwise insert it.
        span.path.as_ref().map(|path_buf| {
            let loc_idx = context
                .metadata
                .iter()
                .find_map(|(idx, md)| match md {
                    Metadatum::FileLocation(file_loc_path_buf, _)
                        if Arc::ptr_eq(path_buf, file_loc_path_buf) =>
                    {
                        Some(MetadataIndex(idx))
                    }
                    _otherwise => None,
                })
                .unwrap_or_else(|| {
                    // This is assuming that the string in this span represents the entire file
                    // found at `path_buf`.
                    MetadataIndex(context.metadata.insert(Metadatum::FileLocation(
                        path_buf.clone(),
                        span.span.input().clone(),
                    )))
                });

            MetadataIndex(context.metadata.insert(Metadatum::Span {
                loc_idx,
                start: span.start(),
                end: span.end(),
            }))
        })
    }

    pub fn to_span(&self, context: &Context) -> Result<Span, IrError> {
        match &context.metadata[self.0] {
            Metadatum::Span {
                loc_idx,
                start,
                end,
            } => {
                let (path, src) = match &context.metadata[loc_idx.0] {
                    Metadatum::FileLocation(path, src) => Ok((path.clone(), src.clone())),
                    _otherwise => Err(IrError::InvalidMetadatum),
                }?;
                Ok(Span {
                    span: pest::Span::new(src, *start, *end).ok_or(IrError::InvalidMetadatum)?,
                    path: Some(path),
                })
            }
            _otherwise => Err(IrError::InvalidMetadatum),
        }
    }
}
