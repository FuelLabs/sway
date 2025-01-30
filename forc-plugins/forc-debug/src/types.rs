use dap::types::Breakpoint;
use std::{collections::HashMap, path::PathBuf};

pub type ExitCode = i64;
pub type Instruction = u64;
pub type Breakpoints = HashMap<PathBuf, Vec<Breakpoint>>;
