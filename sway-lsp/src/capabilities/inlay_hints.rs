use crate::{
    config::InlayHintsConfig,
    core::{
        session::Session,
        token::{TypedAstToken, AstToken, get_range_from_span},
    },
};
use lsp_types::{self, Range, Url};
use std::sync::Arc;
use sway_core::{
    language::ty::{TyDecl, TyVariableDecl, self},
    type_system::TypeInfo, fuel_prelude::fuel_vm::call,
};
use sway_types::Spanned;

// Future PR's will add more kinds
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlayKind {
    TypeHint,
    Parameter,
}

#[derive(Debug)]
pub struct InlayHint {
    pub range: Range,
    pub kind: InlayKind,
    pub label: String,
}

pub fn inlay_hints(
    session: Arc<Session>,
    uri: &Url,
    range: &Range,
    config: &InlayHintsConfig,
) -> Option<Vec<lsp_types::InlayHint>> {
    // 1. Loop through all our tokens and filter out all tokens that aren't TypedVariableDeclaration tokens
    // 2. Also filter out all tokens that have a span that fall outside of the provided range
    // 3. Filter out all variable tokens that have a type_ascription
    // 4. Look up the type id for the remaining tokens
    // 5. Convert the type into a string
    if !config.type_hints {
        return None;
    }

    let engines = session.engines.read();
    let mut hints: Vec<lsp_types::InlayHint> = session
        .token_map()
        .tokens_for_file(uri)
        // Filter out all tokens that have a span that fall outside of the provided range
        .filter_map(|(ident, token)| (ident.range.start >= range.start && ident.range.end <= range.end).then(|| (ident, token)))
        .filter_map(|(ident, token)| {
            // if let AstToken::TypedFunctionApplicationArgument((base_ident, exp)) = token.parsed {
            //     eprintln!("inlay_hints: TypedFunctionApplicationArgument: {:#?}", exp);
            //     eprintln!("range: {:#?}", ident.range);
            //     params::hints(&base_ident, ident.range, &config)
            // } else {
                token.typed.as_ref().and_then(|t| match t {
                    TypedAstToken::TypedDeclaration(TyDecl::VariableDecl(var_decl)) => {
                        var_decl::hints(var_decl, ident.range, &config, &engines)
                    }
                    // TypedAstToken::TypedFunctionApplicationArgument((base_ident, exp)) => {
                    //     eprintln!("inlay_hints: TypedFunctionApplicationArgument: {:#?}", exp);
                    //     eprintln!("range: {:#?}", ident.range);
                    //     params::hints(base_ident, ident.range, &config)
                    // }
                    _ => None,
                })
            //}
        })
        .collect();

    if let Some(ty_program) = &session.compiled_program.read().typed {
        hints.extend(parse(&ty_program, config));
    }

    Some(hints)
}

fn inlay_hint(render_colons: bool, inlay_hint: InlayHint) -> lsp_types::InlayHint {
    lsp_types::InlayHint {
        position: match inlay_hint.kind {
            InlayKind::TypeHint => inlay_hint.range.end,
            InlayKind::Parameter => inlay_hint.range.start,
        },
        label: lsp_types::InlayHintLabel::String(
            if render_colons {
                match inlay_hint.kind { 
                    InlayKind::TypeHint => format!(": {}", inlay_hint.label),
                    InlayKind::Parameter => format!("{}:", inlay_hint.label),
                }
        } else {
            inlay_hint.label
        }),
        kind: match inlay_hint.kind {
            InlayKind::TypeHint => Some(lsp_types::InlayHintKind::TYPE),
            InlayKind::Parameter => Some(lsp_types::InlayHintKind::PARAMETER),
        },
        tooltip: None,
        padding_left: Some(!render_colons),
        padding_right: Some(true),
        text_edits: None,
        data: None,
    }
}

fn test() {
    let y = my_func_addr(400, false);
}

fn my_func_addr(x: u32, b: bool) {
    x + 1;
}

mod var_decl {
    use sway_core::Engines;
    use super::*;

    pub fn hints(
        var_decl: &Box<TyVariableDecl>,
        range: Range,
        config: &InlayHintsConfig,
        engines: &Engines,
    ) -> Option<lsp_types::InlayHint> {
        if var_decl.type_ascription.call_path_tree.is_some() {
            return None;
        }
        match engines.te().get(var_decl.type_ascription.type_id) {
            TypeInfo::Unknown | TypeInfo::UnknownGeneric { .. } => None,
            _ => {
                let label = engines.help_out(&var_decl.type_ascription).to_string();
                let inlay_hint = InlayHint {
                    range,
                    kind: InlayKind::TypeHint,
                    label,
                };
                Some(self::inlay_hint(config.render_colons, inlay_hint))
            }
        }
    }
}

mod params {
    use sway_core::{Engines, language::ty::{TyFunctionParameter, TyExpression}};
    use sway_types::Ident;
    use super::*;

    // find an AmbiguousPathExpression ast token
    // iter over the arguments of this type.
    // use the span of these arguments to somehow "lookup" the typed versions of these arguments
    // get the type_id of the typed version of the argument

    pub fn hints(
        name: &Ident,
        range: Range,
        config: &InlayHintsConfig,
    ) -> Option<lsp_types::InlayHint> {
        let label = name.as_str().to_string();
        let inlay_hint = InlayHint {
            range,
            kind: InlayKind::Parameter,
            label,
        };
        Some(self::inlay_hint(config.render_colons, inlay_hint))
    }
}









// fn parse(ty_program: &ty::TyProgram, config: &InlayHintsConfig) -> Vec<lsp_types::InlayHint> {
//     let root_nodes = ty_program.root.all_nodes.iter();
//     let sub_nodes = ty_program
//         .root
//         .submodules
//         .iter()
//         .flat_map(|(_, submodule)| submodule.module.all_nodes.iter());

//     root_nodes.chain(sub_nodes).map(|n| {
//         if let ty::TyAstNodeContent::Expression(exp) = &n.content {
//             if let ty::TyExpressionVariant::FunctionApplication {
//                 call_path,
//                 contract_call_params,
//                 arguments,
//                 fn_ref,
//                 type_binding,
//                 call_path_typeid,
//                 ..
//             } = &exp.expression
//             {
//                 eprintln!("call_path: {:#?}", call_path);
//                 eprintln!("contract_call_params: {:#?}", contract_call_params);
//                 eprintln!("arguments: {:#?}", arguments);
//                 eprintln!("fn_ref: {:#?}", fn_ref);
//                 eprintln!("type_binding: {:#?}", type_binding);
//                 eprintln!("call_path_typeid: {:#?}", call_path_typeid);

//                 for (ident, exp) in arguments {
//                     if call_path.suffix.name_override_opt().is_none() {
//                         if let ty::TyExpressionVariant::FunctionApplication { call_path, .. } = &exp.expression {
//                             params::hints(ident.as_str().to_string(), get_range_from_span(call_path.span()), config)
//                         }
//                     }
//                 }
//             }
//         }
//     }).collect()
// }

fn parse(ty_program: &ty::TyProgram, config: &InlayHintsConfig) -> Vec<lsp_types::InlayHint> {
    let root_nodes = ty_program.root.all_nodes.iter();
    let sub_nodes = ty_program
        .root
        .submodules
        .iter()
        .flat_map(|(_, submodule)| submodule.module.all_nodes.iter());

    root_nodes.chain(sub_nodes)
        .filter_map(|n| {
            if let ty::TyAstNodeContent::Expression(exp) = &n.content {
                if let ty::TyExpressionVariant::FunctionApplication {
                    call_path,
                    contract_call_params,
                    arguments,
                    fn_ref,
                    type_binding,
                    call_path_typeid,
                    ..
                } = &exp.expression
                {
                    // Here you might want to debug-print information
                    // eprintln!("call_path: {:#?}", call_path);
                    // ...

                    // Process the arguments and collect hints
                    Some(arguments.iter().filter_map(|(ident, exp)| {
                        if call_path.suffix.name_override_opt().is_none() {
                            if let ty::TyExpressionVariant::FunctionApplication { call_path, .. } = &exp.expression {
                                Some(params::hints(ident, get_range_from_span(&call_path.suffix.span()), config))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .flatten() // Flatten the Vec<Option<Vec<InlayHint>>> into an Iterator<Item = Vec<InlayHint>>
        .flatten() // Flatten the Vec<InlayHint> into an Iterator<Item = InlayHint>
        .collect() // Collect the InlayHint items into a Vec<InlayHint>
}
