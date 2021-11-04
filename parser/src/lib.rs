mod lexer;
mod span;
mod token;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream};
use generational_arena::Index;
use std::{collections::HashMap, env, fmt, fs};

pub(crate) use lexer::*;
pub(crate) use span::*;
pub(crate) use token::*;
