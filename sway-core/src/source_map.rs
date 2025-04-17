use dirs::home_dir;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use sway_types::{LineCol, SourceEngine};

use serde::{Deserialize, Serialize};

use sway_types::span::Span;

/// Index of an interned path string
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PathIndex(pub usize);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceMap {
    /// Paths of dependencies in the `~/.forc` directory, with the prefix stripped.
    /// This makes inverse source mapping work on any machine with deps downloaded.
    pub dependency_paths: Vec<PathBuf>,
    /// Paths to source code files, defined separately to avoid repetition.
    pub paths: Vec<PathBuf>,
    /// Mapping from opcode index to source location
    // count of instructions, multiply the opcode by 4 to get the byte offset
    pub map: BTreeMap<usize, SourceMapSpan>,
}
impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts dependency path. Unsupported locations are ignored for now.
    pub fn insert_dependency<P: AsRef<Path>>(&mut self, path: P) {
        if let Some(home) = home_dir() {
            let forc = home.join(".forc/");
            if let Ok(unprefixed) = path.as_ref().strip_prefix(forc) {
                self.dependency_paths.push(unprefixed.to_owned());
            }
        }
        // TODO: Only dependencies in ~/.forc are supported for now
    }

    pub fn insert(&mut self, source_engine: &SourceEngine, pc: usize, span: &Span) {
        if let Some(source_id) = span.source_id() {
            let path = source_engine.get_path(source_id);
            let path_index = self
                .paths
                .iter()
                .position(|p| *p == *path)
                .unwrap_or_else(|| {
                    self.paths.push((*path).to_owned());
                    self.paths.len() - 1
                });
            self.map.insert(
                pc,
                SourceMapSpan {
                    path: PathIndex(path_index),
                    range: LocationRange {
                        start: span.start_line_col_one_index(),
                        end: span.end_line_col_one_index(),
                    },
                },
            );
        }
    }

    /// Inverse source mapping
    pub fn addr_to_span(&self, pc: usize) -> Option<(PathBuf, LocationRange)> {
        self.map
            .get(&pc)
            .map(|sms| sms.to_span(&self.paths, &self.dependency_paths))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapSpan {
    pub path: PathIndex,
    pub range: LocationRange,
}

impl SourceMapSpan {
    pub fn to_span(
        &self,
        paths: &[PathBuf],
        dependency_paths: &[PathBuf],
    ) -> (PathBuf, LocationRange) {
        let p = &paths[self.path.0];
        for dep in dependency_paths {
            if p.starts_with(dep.file_name().unwrap()) {
                let mut path = home_dir().expect("Could not get homedir").join(".forc");

                if let Some(dp) = dep.parent() {
                    path = path.join(dp);
                }

                return (path.join(p), self.range);
            }
        }

        (p.to_owned(), self.range)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LocationRange {
    pub start: LineCol,
    pub end: LineCol,
}
