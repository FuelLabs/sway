use dap::types::Breakpoint;
use std::collections::HashMap;
use std::path::PathBuf;

pub type DynResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type Line = i64;
pub type Instruction = u64;
pub type FileSourceMap = HashMap<Line, Vec<Instruction>>;
pub type SourceMap = HashMap<PathBuf, FileSourceMap>;
pub type Breakpoints = HashMap<PathBuf, Vec<Breakpoint>>;
