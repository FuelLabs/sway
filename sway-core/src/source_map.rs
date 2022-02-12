use dirs::home_dir;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use sway_types::span::Span;

/// Index of an interned path string
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PathIndex(usize);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceMap {
    /// Paths of dependencies in the `~/.forc` directory, with the prefix stripped.
    /// This makes inverse source mapping work on any machine with deps downloaded.
    dependency_paths: Vec<PathBuf>,
    /// Paths to source code files, defined separately to avoid repetition.
    paths: Vec<PathBuf>,
    /// Mapping from opcode index to source location
    map: HashMap<usize, SourceMapSpan>,
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

    /// Inverse source mapping
    pub fn addr_to_span(&self, pc: usize) -> Option<(PathBuf, LocationRange)> {
        self.map.get(&pc).map(|sms| {
            let p = &self.paths[sms.path.0];
            for dep in &self.dependency_paths {
                if p.starts_with(dep.file_name().unwrap()) {
                    let mut path = home_dir().expect("Could not get homedir").join(".forc");

                    if let Some(dp) = dep.parent() {
                        path = path.join(dp);
                    }

                    return (path.join(p), sms.range);
                }
            }

            (p.to_owned(), sms.range)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapSpan {
    pub path: PathIndex,
    pub range: LocationRange,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LocationRange {
    pub start: usize,
    pub end: usize,
}
