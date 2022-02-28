//! Used to debug function selectors
//! Given an input function declaration, return the selector for it in hexidecimal.
use clap::Parser as ClapParser;
use pest::Parser;
use std::sync::Arc;
use sway_core::{
    create_module,
    parse_tree::declaration::FunctionDeclaration,
    semantic_analysis::{
        ast_node::{declaration::TypedFunctionDeclaration, impl_trait::Mode},
        TypeCheckArguments,
    },
    type_engine::*,
    BuildConfig, Rule, SwayParser,
};

#[derive(Debug, ClapParser)]
#[clap(name = "example", about = "An example of Clap Parser usage.")]
struct Opt {
    fn_decl: String,
}

fn main() {
    let opt = Opt::parse();
    let fn_decl = opt.fn_decl;
    let mut warnings = vec![];
    let mut errors = vec![];

    let parsed_fn_decl = SwayParser::parse(Rule::fn_decl, Arc::from(fn_decl));
    let mut parsed_fn_decl = match parsed_fn_decl {
        Ok(o) => o,
        Err(e) => panic!("Failed to parse: {:?}", e),
    };
    let parsed_fn_decl =
        FunctionDeclaration::parse_from_pair(parsed_fn_decl.next().unwrap(), Default::default())
            .unwrap(&mut warnings, &mut errors);

    let namespace = create_module();
    let res = TypedFunctionDeclaration::type_check(TypeCheckArguments {
        checkee: parsed_fn_decl,
        namespace,
        crate_namespace: namespace,
        help_text: "",
        return_type_annotation: insert_type(TypeInfo::Unknown),
        self_type: insert_type(TypeInfo::Unknown),
        build_config: &mut BuildConfig::root_from_file_name_and_manifest_path(
            Default::default(),
            Default::default(),
        ),
        dead_code_graph: &mut Default::default(),
        mode: Mode::ImplAbiFn,
        opts: Default::default(),
    })
    .unwrap(&mut warnings, &mut errors);

    let selector_string = res.to_selector_name().unwrap(&mut warnings, &mut errors);
    let selector_hash_untruncated = res
        .to_fn_selector_value_untruncated()
        .unwrap(&mut warnings, &mut errors);
    let selector_hash_untruncated = hex::encode(selector_hash_untruncated);
    let selector_hash = res
        .to_fn_selector_value()
        .unwrap(&mut warnings, &mut errors);
    let selector_hash = hex::encode(selector_hash);
    println!("selector string:         {}", selector_string);
    println!("untruncated hash:        0x{}", selector_hash_untruncated);
    println!("truncated/padded hash:   0x00000000{}", selector_hash);
}
