use anyhow::{anyhow, Result};
use clap::Parser;
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader};
use std::path::{Path, PathBuf};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{AnnotationType, Slice, Snippet, SourceAnnotation},
};

use sway_core::source_map::{LocationRange, SourceMap};

/// Show location and context of an opcode address in its source file
#[derive(Debug, Parser)]
pub(crate) struct Command {
    /// Where to search for the project root
    #[clap(short = 's', long, default_value = ".")]
    pub search_dir: PathBuf,
    /// Source file mapping in JSON format
    #[clap(short = 'g', long)]
    pub sourcemap_path: PathBuf,
    /// How many lines of context to show
    #[clap(short, long, default_value = "2")]
    pub context: usize,
    /// Opcode index
    #[clap(short = 'i', long)]
    pub opcode_index: usize,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    let contents = fs::read(&command.sourcemap_path)
        .map_err(|err| anyhow!("{:?}: could not read: {:?}", command.sourcemap_path, err))?;

    let sm: SourceMap = serde_json::from_slice(&contents).map_err(|err| {
        anyhow!(
            "{:?}: invalid source map json: {}",
            command.sourcemap_path,
            err
        )
    })?;

    if let Some((mut path, range)) = sm.addr_to_span(command.opcode_index) {
        if path.is_relative() {
            path = command.search_dir.join(path);
        }

        let rr = read_range(&path, range, command.context)
            .map_err(|err| anyhow!("{:?}: could not read: {:?}", path, err))?;

        let path_str = format!("{:?}", path);
        let snippet = Snippet {
            title: None,
            footer: vec![],
            slices: vec![Slice {
                source: &rr.source,
                line_start: rr.source_start_line,
                origin: Some(&path_str),
                fold: false,
                annotations: vec![SourceAnnotation {
                    label: "here",
                    annotation_type: AnnotationType::Note,
                    range: (rr.offset, rr.offset + rr.length),
                }],
            }],
            opt: FormatOptions {
                color: true,
                ..Default::default()
            },
        };
        println!("{}", DisplayList::from(snippet));

        Ok(())
    } else {
        Err(anyhow!("Address did not map to any source code location"))
    }
}

struct ReadRange {
    source: String,
    source_start_byte: usize,
    source_start_line: usize,
    offset: usize,
    length: usize,
}

fn read_range<P: AsRef<Path>>(
    path: P,
    range: LocationRange,
    context_lines: usize,
) -> io::Result<ReadRange> {
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut context_buffer = VecDeque::new();

    let mut start_pos = None;
    let mut position = 0;
    for line_num in 0.. {
        let mut buffer = String::new();
        let n = reader.read_line(&mut buffer)?;
        context_buffer.push_back(buffer);
        if start_pos.is_none() {
            if position + n > range.start {
                let cbl: usize = context_buffer.iter().map(|c| c.len()).sum();
                start_pos = Some((line_num, position, range.start - (position + n - cbl)));
            } else if context_buffer.len() > context_lines {
                let _ = context_buffer.pop_front();
            }
        } else if context_buffer.len() > context_lines * 2 {
            break;
        }

        position += n;
    }

    let source = context_buffer.make_contiguous().join("");
    let length = range.end - range.start;

    let (source_start_line, source_start_byte, offset) = start_pos.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Source file was modified, and the mapping is now out of range",
        )
    })?;

    if offset + length > source.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Source file was modified, and the mapping is now out of range",
        ));
    }

    Ok(ReadRange {
        source,
        source_start_byte,
        source_start_line,
        offset,
        length,
    })
}
