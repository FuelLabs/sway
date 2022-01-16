use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use sway_types::span::Span;

/// Index of an interned path string
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PathIndex(usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    paths: Vec<PathBuf>,
    map: HashMap<usize, SourceMapSpan>,
}
impl SourceMap {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, pc: usize, span: &Span) {
        if let Some(path) = span.path.as_ref() {
            let path_index = self
                .paths
                .iter()
                .position(|p| *p == **path)
                .unwrap_or_else(|| {
                    self.paths.push((**path).to_owned());
                    self.paths.len() - 1
                });
            self.map.insert(
                pc,
                SourceMapSpan {
                    path: PathIndex(path_index),
                    range: LocationRange {
                        start: span.start(),
                        end: span.end(),
                    },
                },
            );
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapSpan {
    pub path: PathIndex,
    pub range: LocationRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRange {
    pub start: usize,
    pub end: usize,
}
