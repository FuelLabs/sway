use core_lang::HllParseTree;

mod builder;
mod common;
mod traversal;

pub fn build_from_tree(parse_tree: HllParseTree) -> Result<(), String> {
    traversal::traverse_and_build(parse_tree)
}
