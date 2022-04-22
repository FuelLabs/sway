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
        span.path().map(|path_buf| {
            let loc_idx = match context.metadata_reverse_map.get(&Arc::as_ptr(path_buf)) {
                Some(idx) => *idx,
                None => {
                    // This is assuming that the string in this span represents the entire file
                    // found at `path_buf`.
                    let new_idx = MetadataIndex(context.metadata.insert(Metadatum::FileLocation(
                        path_buf.clone(),
                        span.src().clone(),
                    )));
                    context
                        .metadata_reverse_map
                        .insert(Arc::as_ptr(path_buf), new_idx);
                    new_idx
                }
            };

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
                Span::new(src, *start, *end, Some(path)).ok_or(IrError::InvalidMetadatum)
            }
            _otherwise => Err(IrError::InvalidMetadatum),
        }
    }
}
