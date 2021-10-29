use crate::utils::helpers::{
    format_err, format_warning, println_green_err, println_red_err, println_yellow_err,
};
use core_lang::{CompileError, CompileWarning};

pub fn aborting_due_to<'sc>(
    silent_mode: bool,
    warnings: Vec<CompileWarning<'sc>>,
    errors: Vec<CompileError<'sc>>,
) {
    let e_len = errors.len();

    if !silent_mode {
        warnings.iter().for_each(|warning| format_warning(warning));
        errors.into_iter().for_each(|error| format_err(&error));
    }

    println_red_err(&format!(
        "  Aborting due to {} {}.",
        e_len,
        if e_len > 1 { "errors" } else { "error" }
    ))
    .unwrap();
}

pub fn compiled_with_warnings<'sc>(
    silent_mode: bool,
    proj_name: String,
    warnings: Vec<CompileWarning<'sc>>,
) {
    if !silent_mode {
        warnings.iter().for_each(|warning| format_warning(warning));
    }

    if warnings.is_empty() {
        let _ = println_green_err(&format!("  Compiled library {:?}.", proj_name));
    } else {
        let _ = println_yellow_err(&format!(
            "  Compiled library {:?} with {} {}.",
            proj_name,
            warnings.len(),
            if warnings.len() > 1 {
                "warnings"
            } else {
                "warning"
            }
        ));
    }
}
