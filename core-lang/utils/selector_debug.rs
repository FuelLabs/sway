//! Used to debug function selectors
//! Given an input function declaration, return the selector for it in hexidecimal.
use fuel_pest::Parser;
use structopt::StructOpt;

use core_lang::{
    error::CompileResult,
    parse_tree::function_declaration::FunctionDeclaration,
    semantic_analysis::ast_node::{declaration::TypedFunctionDeclaration, impl_trait::Mode},
    type_engine::TypeInfo,
    BuildConfig, HllParser, Rule,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    fn_decl: String,
}

fn main() {
    let opt = Opt::from_args();
    let fn_decl = opt.fn_decl;

    let parsed_fn_decl = HllParser::parse(Rule::fn_decl, &fn_decl);
    let mut parsed_fn_decl = match parsed_fn_decl {
        Ok(o) => o,
        Err(e) => panic!("Failed to parse: {:?}", e),
    };
    let parsed_fn_decl = FunctionDeclaration::parse_from_pair(parsed_fn_decl.next().unwrap());
    let parsed_fn_decl = match parsed_fn_decl {
        CompileResult::Ok { value, .. } => value,
        CompileResult::Err { errors, .. } => panic!("Failed to parse: {:?}", errors),
    };

    let res = match TypedFunctionDeclaration::type_check(
        parsed_fn_decl,
        &Default::default(),
        None,
        "",
        TypeInfo::Unit,
        &BuildConfig::root_from_manifest_path(Default::default()),
        &mut Default::default(),
        Mode::ImplAbiFn,
    ) {
        CompileResult::Ok { value, .. } => value,
        CompileResult::Err { errors, .. } => panic!("Failed to type check: {:?}", errors),
    };

    let selector_string = match res.to_selector_name() {
        CompileResult::Ok { value, .. } => value,
        CompileResult::Err { errors, .. } => {
            panic!("Failed to construct selector name: {:?}", errors)
        }
    };
    let selector_hash_untruncated = match res.to_fn_selector_value_untruncated() {
        CompileResult::Ok { value, .. } => value,
        CompileResult::Err { errors, .. } => panic!("Failed to construct hash: {:?}", errors),
    };
    let selector_hash_untruncated = hex::encode(selector_hash_untruncated);
    let selector_hash = match res.to_fn_selector_value() {
        CompileResult::Ok { value, .. } => value,
        CompileResult::Err { errors, .. } => panic!("Failed to construct hash: {:?}", errors),
    };
    let selector_hash = hex::encode(selector_hash);
    println!("selector string:         {}", selector_string);
    println!("untruncated hash:        0x{}", selector_hash_untruncated);
    println!("truncated/padded hash:   0x00000000{}", selector_hash);
}
