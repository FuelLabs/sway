use std::collections::HashMap;

use sway_core::language::ty::TyStructDeclaration;
use tower_lsp::lsp_types::{CodeActionOrCommand, TextEdit, Url};

use crate::capabilities::code_actions::{build_code_action, range_after_last_line, TAB};

const CODE_ACTION_DESCRIPTION: &str = "Generate impl for struct";

pub(crate) fn code_action(decl: &TyStructDeclaration, uri: &Url) -> CodeActionOrCommand {
    let text_edit = TextEdit {
        range: range_after_last_line(&decl.span),
        new_text: get_impl_string(decl),
    };

    build_code_action(
        CODE_ACTION_DESCRIPTION.to_string(),
        HashMap::from([(uri.clone(), vec![text_edit])]),
        &uri,
    )
}

fn get_impl_string(struct_decl: &TyStructDeclaration) -> String {
    let struct_name = struct_decl.call_path.suffix.as_str();
    format!("\nimpl {} {{\n{}\n}}\n", struct_name, TAB)
}
