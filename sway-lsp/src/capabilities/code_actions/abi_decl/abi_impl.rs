use crate::capabilities::code_actions::{
    common::generate_impl::{GenerateImplCodeAction, CONTRACT},
    CodeAction, CodeActionContext, CODE_ACTION_IMPL_TITLE,
};
use lsp_types::{Range, Url};
use sway_core::{
    language::ty::{self, TyAbiDecl, TyFunctionParameter, TyTraitFn},
    Engines,
};
use sway_types::Spanned;

pub(crate) struct AbiImplCodeAction<'a> {
    engines: &'a Engines,
    decl: &'a TyAbiDecl,
    uri: &'a Url,
}

impl<'a> GenerateImplCodeAction<'a, TyAbiDecl> for AbiImplCodeAction<'a> {
    fn decl_name(&self) -> String {
        self.decl.name.to_string()
    }
}

impl<'a> CodeAction<'a, TyAbiDecl> for AbiImplCodeAction<'a> {
    fn new(ctx: &CodeActionContext<'a>, decl: &'a TyAbiDecl) -> Self {
        Self {
            engines: ctx.engines,
            decl,
            uri: ctx.uri,
        }
    }

    fn new_text(&self) -> String {
        self.impl_string(
            None,
            self.fn_signatures_string(),
            Some(CONTRACT.to_string()),
        )
    }

    fn title(&self) -> String {
        format!("{} `{}`", CODE_ACTION_IMPL_TITLE, self.decl_name())
    }

    fn range(&self) -> Range {
        self.range_after()
    }

    fn decl(&self) -> &TyAbiDecl {
        self.decl
    }

    fn uri(&self) -> &Url {
        self.uri
    }
}

impl AbiImplCodeAction<'_> {
    fn return_type_string(&self, function_decl: &TyTraitFn) -> String {
        let type_engine = self.engines.te();
        // Unit is the implicit return type for ABI functions.
        if type_engine.get(function_decl.return_type.type_id).is_unit() {
            String::new()
        } else {
            format!(" -> {}", function_decl.return_type.span().as_str())
        }
    }

    fn fn_signatures_string(&self) -> String {
        let decl_engine = self.engines.de();
        format!(
            "\n{}\n",
            self.decl
                .interface_surface
                .iter()
                .map(|item| {
                    match item {
                        ty::TyTraitInterfaceItem::TraitFn(function_decl_ref) => {
                            let function_decl = decl_engine.get_trait_fn(function_decl_ref);
                            self.fn_signature_string(
                                function_decl.name.to_string(),
                                params_string(&function_decl.parameters),
                                &function_decl.attributes,
                                self.return_type_string(&function_decl),
                                None,
                            )
                        }
                        ty::TyTraitInterfaceItem::Constant(_)
                        | ty::TyTraitInterfaceItem::Type(_) => unreachable!(),
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

fn params_string(params: &[TyFunctionParameter]) -> String {
    params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.type_argument.span().as_str()))
        .collect::<Vec<String>>()
        .join(", ")
}
