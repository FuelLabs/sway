use sway_ast::{attribute::Annotated, Module};
use sway_features::ExperimentalFeatures;
use sway_parse::parse_file as sway_parse_parse_file;

pub fn parse_file(src: &str) -> Option<Annotated<Module>> {
    let handler = <_>::default();
    let path = None;
    sway_parse_parse_file(&handler, src.into(), path, ExperimentalFeatures::default()).ok()
}
