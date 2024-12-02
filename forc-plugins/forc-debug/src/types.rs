use dap::types::Breakpoint;
use std::{collections::HashMap, path::PathBuf};

pub type Line = i64;
pub type Instruction = u64;
pub type FileSourceMap = HashMap<Line, Vec<Instruction>>;
pub type SourceMap = HashMap<PathBuf, FileSourceMap>;
pub type Breakpoints = HashMap<PathBuf, Vec<Breakpoint>>;
