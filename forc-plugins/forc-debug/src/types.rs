use std::collections::HashMap;
use std::path::PathBuf;

pub type DynResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type Line = i64;
pub type Instruction = u64;
pub type SourceMap = HashMap<PathBuf, HashMap<Line, Instruction>>;
