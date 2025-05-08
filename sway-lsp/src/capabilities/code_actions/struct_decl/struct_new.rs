use crate::{
    capabilities::code_actions::{
        common::generate_impl::GenerateImplCodeAction, CodeAction, CodeActionContext,
        CODE_ACTION_NEW_TITLE,
    },
    core::{token::TypedAstToken, token_map::TokenMapExt},
};
use lsp_types::{CodeActionDisabled, Position, Range, Url};
use sway_core::language::ty::{self, TyImplSelfOrTrait, TyStructDecl, TyStructField};
use sway_types::{LineCol, Spanned};

pub(crate) struct StructNewCodeAction<'a> {
    decl: &'a TyStructDecl,
    uri: &'a Url,
    existing_impl_decl: Option<TyImplSelfOrTrait>,
}

impl<'a> GenerateImplCodeAction<'a, TyStructDecl> for StructNewCodeAction<'a> {
    fn decl_name(&self) -> String {
        self.decl.call_path.suffix.to_string()
    }
}

impl<'a> CodeAction<'a, TyStructDecl> for StructNewCodeAction<'a> {
    fn new(ctx: &CodeActionContext<'a>, decl: &'a TyStructDecl) -> Self {
        // Before the other functions are called, we need to determine if the new function
        // should be generated in a new impl block, an existing impl block, or not at all.
        // Find the first impl block for this struct if it exists.
        let existing_impl_decl = ctx
            .tokens
            .iter()
            .all_references_of_token(ctx.token, ctx.engines)
            .find_map(|item| {
                if let Some(TypedAstToken::TypedDeclaration(ty::TyDecl::ImplSelfOrTrait(
                    ty::ImplSelfOrTrait { decl_id, .. },
                ))) = item.value().as_typed()
                {
                    Some((*ctx.engines.de().get_impl_self_or_trait(decl_id)).clone())
                } else {
                    None
                }
            });
        Self {
            decl,
            uri: ctx.uri,
            existing_impl_decl,
        }
    }

    fn new_text(&self) -> String {
        let params = StructNewCodeAction::params_string(&self.decl.fields);
        let new_fn = self.fn_signature_string(
            "new".to_string(),
            params,
            &self.decl.attributes,
            self.return_type_string(),
            Some(self.fn_body()),
        );

        // If there is already an impl block for this struct, add only the function to it.
        if self.existing_impl_decl.is_some() {
            format!("{new_fn}\n")
        } else {
            // Otherwise, generate the impl block with the `new` function inside.
            self.impl_string(
                self.type_param_string(&self.decl.generic_parameters),
                format!("\n{new_fn}\n"),
                None,
            )
        }
    }

    fn range(&self) -> Range {
        // If there is already an impl block for this struct, insert the new function at the top of it.
        let insertion_position = match self.existing_impl_decl.clone() {
            Some(decl) => {
                let LineCol {
                    line: first_line, ..
                } = decl.span.start_line_col_one_index();
                Position {
                    line: first_line as u32,
                    character: 0,
                }
            }
            None => {
                // If we're inserting a whole new impl block, default to the line after the struct declaration.
                let LineCol {
                    line: last_line, ..
                } = self.decl().span().end_line_col_one_index();
                Position {
                    line: last_line as u32,
                    character: 0,
                }
            }
        };
        Range {
            start: insertion_position,
            end: insertion_position,
        }
    }

    fn title(&self) -> String {
        CODE_ACTION_NEW_TITLE.to_string()
    }

    fn decl(&self) -> &TyStructDecl {
        self.decl
    }

    fn uri(&self) -> &Url {
        self.uri
    }

    fn disabled(&self) -> Option<CodeActionDisabled> {
        // If there is already a `new` function in the impl block, don't generate a new one.
        if self
            .existing_impl_decl
            .clone()?
            .items
            .iter()
            .any(|item| match item {
                sway_core::language::ty::TyTraitItem::Fn(fn_decl) => {
                    fn_decl.span().as_str().contains("fn new")
                }
                sway_core::language::ty::TyTraitItem::Constant(_) => false,
                sway_core::language::ty::TyTraitItem::Type(_) => false,
            })
        {
            Some(CodeActionDisabled {
                reason: format!("Struct {} already has a `new` function", self.decl_name()),
            })
        } else {
            None
        }
    }
}

impl StructNewCodeAction<'_> {
    fn return_type_string(&self) -> String {
        " -> Self".to_string()
    }

    fn params_string(params: &[TyStructField]) -> String {
        params
            .iter()
            .map(|field| format!("{}: {}", field.name, field.type_argument.span().as_str()))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn fn_body(&self) -> String {
        if self.decl.fields.is_empty() {
            return "Self {{}}".to_string();
        }
        format!(
            "Self {{ {} }}",
            self.decl
                .fields
                .iter()
                .map(|field| format!("{}", field.name))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
