//! Used to debug function selectors
//! Given an input function declaration, return the selector for it in hexidecimal.
use pest::Parser;
use std::path::PathBuf;
use structopt::StructOpt;

use core_lang::{HllParser, Rule};

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    fn_decl: String,
}

fn main() {
    let opt = Opt::from_args();
    let fn_decl = opt.fn_decl;

    let parsed_fn_decl = HllParser::parse(Rule::fn_decl, &fn_decl);
    let parsed_fn_decl = match parsed_fn_decl {
        Ok(o) => o,
        Err(e) => panic!("Failed to parse: {:?}", e),
    };
}
