use crate::abi_spec;
use crate::cli::AbiSpecCommand;
use crate::utils::errors::{aborting_due_to, compiled_with_warnings};

use crate::utils::dependency::{Dependency, DependencyDetails};
use crate::{
    cli::BuildCommand,
    utils::dependency,
    utils::helpers::{
        find_manifest_dir, format_err, format_warning, get_file_name, get_main_file, get_main_path,
        println_green_err, println_red_err, println_yellow_err, read_manifest,
    },
};
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use core_lang::{
    BuildConfig, BytecodeCompilationResult, CompilationResult, CompileResult, LibraryExports,
    Namespace, TypedCompilationResult,
};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub fn generate_abi_spec(command: AbiSpecCommand) -> Result<Vec<u8>, String> {
    let AbiSpecCommand {
        path,
        offline_mode,
        silent_mode,
        json_outfile,
    } = command;

    unimplemented!()
}

fn generate_abi_spec_main<'source, 'manifest>(
    source: &'source str,
    proj_name: &str,
    namespace: &Namespace<'source>,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    silent_mode: bool,
) -> Result<Value, String> {
    let res = core_lang::compile_to_typed_ast(&source, namespace, &build_config, dependency_graph);
    match res {
        TypedCompilationResult::Success { ast, warnings } => {
            let CompileResult {
                value,
                warnings,
                errors,
            } = abi_spec::generate_abi_spec(ast);
            match value {
                Some(value) => {
                    compiled_with_warnings(silent_mode, proj_name.to_string(), warnings);
                    return Ok(value);
                }
                None => {
                    aborting_due_to(silent_mode, warnings, errors);
                    return Err(format!("Failed to generate abi spec for {}", proj_name));
                }
            }
        }
        TypedCompilationResult::Library { warnings, .. } => {
            compiled_with_warnings(silent_mode, proj_name.to_string(), warnings);
            return Ok(json!("".to_string()));
        }
        TypedCompilationResult::Failure { errors, warnings } => {
            aborting_due_to(silent_mode, warnings, errors);
            return Err(format!("Failed to compile {}", proj_name));
        }
    }
}
