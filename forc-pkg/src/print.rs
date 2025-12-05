//! Printing utilities for compilation status and diagnostics.

use forc_diagnostic::{
    format_diagnostic, println_action_green, println_red_err, println_yellow_err,
};
use sway_core::language::parsed::TreeType;
use sway_error::{
    diagnostic::ToDiagnostic,
    error::CompileError,
    warning::{CompileInfo, CompileWarning},
};
use sway_types::SourceEngine;

pub fn program_type_str(ty: &TreeType) -> &'static str {
    match ty {
        TreeType::Script => "script",
        TreeType::Contract => "contract",
        TreeType::Predicate => "predicate",
        TreeType::Library => "library",
    }
}

pub fn print_compiling(ty: Option<&TreeType>, name: &str, src: &dyn std::fmt::Display) {
    // NOTE: We can only print the program type if we can parse the program, so
    // program type must be optional.
    let ty = match ty {
        Some(ty) => format!("{} ", program_type_str(ty)),
        None => "".to_string(),
    };
    println_action_green(
        "Compiling",
        &format!("{ty}{} ({src})", ansiterm::Style::new().bold().paint(name)),
    );
}

pub fn print_infos(source_engine: &SourceEngine, terse_mode: bool, infos: &[CompileInfo]) {
    if infos.is_empty() {
        return;
    }

    if !terse_mode {
        infos
            .iter()
            .for_each(|n| format_diagnostic(&n.to_diagnostic(source_engine)));
    }
}

pub fn print_warnings(
    source_engine: &SourceEngine,
    terse_mode: bool,
    proj_name: &str,
    warnings: &[CompileWarning],
    tree_type: &TreeType,
) {
    if warnings.is_empty() {
        return;
    }
    let type_str = program_type_str(tree_type);

    if !terse_mode {
        warnings
            .iter()
            .for_each(|w| format_diagnostic(&w.to_diagnostic(source_engine)));
    }

    println_yellow_err(&format!(
        "  Compiled {} {:?} with {} {}.",
        type_str,
        proj_name,
        warnings.len(),
        if warnings.len() > 1 {
            "warnings"
        } else {
            "warning"
        }
    ));
}

pub fn print_on_failure(
    source_engine: &SourceEngine,
    terse_mode: bool,
    infos: &[CompileInfo],
    warnings: &[CompileWarning],
    errors: &[CompileError],
    reverse_results: bool,
) {
    print_infos(source_engine, terse_mode, infos);

    let e_len = errors.len();
    let w_len = warnings.len();

    if !terse_mode {
        if reverse_results {
            warnings
                .iter()
                .rev()
                .for_each(|w| format_diagnostic(&w.to_diagnostic(source_engine)));
            errors
                .iter()
                .rev()
                .for_each(|e| format_diagnostic(&e.to_diagnostic(source_engine)));
        } else {
            warnings
                .iter()
                .for_each(|w| format_diagnostic(&w.to_diagnostic(source_engine)));
            errors
                .iter()
                .for_each(|e| format_diagnostic(&e.to_diagnostic(source_engine)));
        }
    }

    if e_len == 0 && w_len > 0 {
        println_red_err(&format!(
            "  Aborting. {} warning(s) treated as error(s).",
            warnings.len()
        ));
    } else {
        println_red_err(&format!(
            "  Aborting due to {} {}.",
            e_len,
            if e_len > 1 { "errors" } else { "error" }
        ));
    }
}
