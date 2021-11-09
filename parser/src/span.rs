//! This module tracks source files and metadata.

use core::ops::Range;
use generational_arena::{Arena, Index};
use lazy_static::lazy_static;

use std::{path::PathBuf, sync::Mutex};

lazy_static! {
    /// Tracks all source code files and metadata associated with them.
    pub static ref SOURCES: Mutex<Arena<SourceFile>> = Default::default();
}

fn insert_source(val: SourceFile) -> Index {
    let mut lock = SOURCES.lock().unwrap();
    let idx = lock.insert(val);
    drop(lock);
    idx
}

fn get_source(ix: Index) -> SourceFile {
    let lock = SOURCES.lock().unwrap();
    let val = (*lock
        .get(ix)
        .expect("Invariant breached: Arena index doesn't exist in arena."))
    .clone();
    drop(lock);
    val
}

/// Represents a Sway source code file.
#[derive(Clone)]
pub struct SourceFile {
    /// The absolute path to the file.
    pub file_path: PathBuf,
    /// The one and only copy in memory of this file's contents as a string.
    /// Only references to this should be distributed to avoid over-cloning.
    pub file_content: String,
}

/// Represents a span of a specific section of source code in a specific file.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    arena_idx: Index,
    range: Range<usize>,
}

impl chumsky::Span for Span {
    // An index to access in `SOURCES`
    type Context = Index;
    type Offset = usize;
    fn new(context: Self::Context, range: Range<Self::Offset>) -> Self {
        Span {
            arena_idx: context,
            range,
        }
    }
    fn context(&self) -> Self::Context {
        todo!()
    }
    fn start(&self) -> Self::Offset {
        todo!()
    }
    fn end(&self) -> Self::Offset {
        todo!()
    }
}

impl Span {
    pub fn index(&self) -> Index {
        self.arena_idx
    }
    pub fn join(&self, other: &Span) -> Span {
        // Only spans from the same file can be joined.
        assert_eq!(
            self.arena_idx, other.arena_idx,
            "Invariant breached: spans from different files cannot be joined"
        );

        let start = self.start();
        let end = self.start();
        Span::new_from_idx(self.arena_idx, start, end)
    }

    /// Constructs a new span that represents an entire file.
    pub fn new_entire_file(new_file_path: PathBuf, file_content: &str) -> Self {
        Self::new_from_file(new_file_path, file_content, 0, file_content.len())
    }
    /// Constructs a new span from a file path and some indexes.
    /// If the file is already in the arena, we reuse the arena index.
    pub fn new_from_file(
        new_file_path: PathBuf,
        file_content: &str,
        start: usize,
        end: usize,
    ) -> Self {
        let sources_lock = SOURCES.lock().unwrap();
        for (idx, SourceFile { ref file_path, .. }) in sources_lock.iter() {
            if *file_path == new_file_path {
                return Span {
                    arena_idx: idx,
                    range: start..end,
                };
            }
        }
        drop(sources_lock);

        Span {
            arena_idx: insert_source(SourceFile {
                file_path: new_file_path,
                file_content: file_content.to_string(),
            }),
            range: start..end,
        }
    }

    pub fn range(&self) -> &Range<<Self as chumsky::Span>::Offset> {
        &self.range
    }

    pub fn new_from_idx(idx: Index, start: usize, end: usize) -> Self {
        Span {
            arena_idx: idx,
            range: start..end,
        }
    }

    pub fn start(&self) -> usize {
        self.range.start
    }

    pub fn end(&self) -> usize {
        self.range.end
    }

    #[cfg(feature = "pest-compat")]
    pub fn start_pos(&self) -> pest::Position {
        let input_file: &SourceFile = get_source(self.arena_idx);
        pest::Position::new(&input_file.file_content, self.range.start)
    }

    #[cfg(feature = "pest-compat")]
    pub fn end_pos<'a>(&self) -> pest::Position<'a> {
        let input_file: &SourceFile = get_source(self.arena_idx);
        pest::Position::new(&input_file.file_content, self.range.end)
    }

    #[cfg(feature = "pest-compat")]
    pub fn split<'a>(&self) -> (pest::Position<'a>, pest::Position<'a>) {
        let input_file: &SourceFile = get_source(self.arena_idx);
        (
            pest::Position::new(&input_file.file_content, self.range.start),
            pest::Position::new(&input_file.file_content, self.range.end),
        )
    }

    pub fn as_string(&self) -> String {
        let input_file: SourceFile = get_source(self.arena_idx);
        input_file.file_content[self.range.start..self.range.end].to_string()
    }

    pub fn input(&self) -> String {
        let input_file: SourceFile = get_source(self.arena_idx);
        input_file.file_content
    }

    pub fn path(&self) -> String {
        let input_file: SourceFile = get_source(self.arena_idx);
        input_file
            .file_path
            .into_os_string()
            .into_string()
            .expect("hopefully the file name isn't invalid utf-8")
    }
}
