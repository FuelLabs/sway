use std::sync::Arc;

use sway_ast::{attribute::Annotated, Module};
use sway_parse::parse_file as sway_parse_parse_file;

pub fn parse_file(input: &str) -> Option<Annotated<Module>> {
    let handler = <_>::default();
    let src = Arc::from(input);
    let path = None;
    sway_parse_parse_file(&handler, src, path).ok()
}
