use sway_ast::ItemConst;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_parse::{lex, Parser};
use sway_types::{constants::CONTRACT_ID, Spanned};

use crate::{
    language::{
        parsed::{AstNode, AstNodeContent, Declaration, ExpressionKind},
        ty::{TyAstNode, TyAstNodeContent},
    },
    semantic_analysis::{TypeCheckContext, namespace::Root},
    transform::to_parsed_lang,
    Engines, Ident, Namespace,
};

///// `contract_id_value` is injected here via forc-pkg when producing the `dependency_namespace` for a contract which has tests enabled.
///// This allows us to provide a contract's `CONTRACT_ID` constant to its own unit tests.
/////
///// This will eventually be refactored out of `sway-core` in favor of creating temporary package dependencies for providing these
///// `CONTRACT_ID`-containing modules: https://github.com/FuelLabs/sway/issues/3077
//pub fn default_with_contract_id(
//    engines: &Engines,
//    name: Ident,
//    contract_id_value: String,
//    experimental: crate::ExperimentalFlags,
//) -> Result<Namespace, vec1::Vec1<CompileError>> {
//    let handler = <_>::default();
//    default_with_contract_id_inner(
//        &handler,
//        engines,
//        name,
//        contract_id_value,
//        experimental,
//    )
//    .map_err(|_| {
//        let (errors, warnings) = handler.consume();
//        assert!(warnings.is_empty());
//
//        // Invariant: `.value == None` => `!errors.is_empty()`.
//        vec1::Vec1::try_from_vec(errors).unwrap()
//    })
//}
//
//fn default_with_contract_id_inner(
//    handler: &Handler,
//    engines: &Engines,
//    ns_name: Ident,
//    contract_id_value: String,
//    experimental: crate::ExperimentalFlags,
//) -> Result<Namespace, ErrorEmitted> {
//    // it would be nice to one day maintain a span from the manifest file, but
//    // we don't keep that around so we just use the span from the generated const decl instead.
//    let mut compiled_constants: SymbolMap = Default::default();
//    // this for loop performs a miniature compilation of each const item in the config
//    // FIXME(Centril): Stop parsing. Construct AST directly instead!
//    // parser config
//    let const_item = format!("pub const CONTRACT_ID: b256 = {contract_id_value};");
//    let const_item_len = const_item.len();
//    let input_arc = std::sync::Arc::from(const_item);
//    let token_stream = lex(handler, &input_arc, 0, const_item_len, None).unwrap();
//    let mut parser = Parser::new(handler, &token_stream);
//    // perform the parse
//    let const_item: ItemConst = parser.parse()?;
//    let const_item_span = const_item.span();
//
//    // perform the conversions from parser code to parse tree types
//    let name = const_item.name.clone();
//    let attributes = Default::default();
//    // convert to const decl
//    let const_decl_id = to_parsed_lang::item_const_to_constant_declaration(
//        &mut to_parsed_lang::Context::new(crate::BuildTarget::EVM, experimental),
//        handler,
//        engines,
//        const_item,
//        attributes,
//        true,
//    )?;
//
//    // Temporarily disallow non-literals. See https://github.com/FuelLabs/sway/issues/2647.
//    let const_decl = engines.pe().get_constant(&const_decl_id);
//    let has_literal = match &const_decl.value {
//        Some(value) => {
//            matches!(value.kind, ExpressionKind::Literal(_))
//        }
//        None => false,
//    };
//
//    if !has_literal {
//        return Err(handler.emit_err(CompileError::ContractIdValueNotALiteral {
//            span: const_item_span,
//        }));
//    }
//
//    let ast_node = AstNode {
//        content: AstNodeContent::Declaration(Declaration::ConstantDeclaration(const_decl_id)),
//        span: const_item_span.clone(),
//    };
//
//    let mut ns = Namespace::new(ns_name, None, true);
//
//    /// CONTINUE HERE
//
//    let mut root = Root::from(Module::new(ns_name.clone(), Visibility::Public, None, vec!()));
//    let mut ns = Namespace::init_root(&mut root);
//    // This is pretty hacky but that's okay because of this code is being removed pretty soon
//    let type_check_ctx = TypeCheckContext::from_namespace(&mut ns, engines, experimental);
//    let typed_node = TyAstNode::type_check(handler, type_check_ctx, &ast_node).unwrap();
//    // get the decl out of the typed node:
//    // we know as an invariant this must be a const decl, as we hardcoded a const decl in
//    // the above `format!`.  if it isn't we report an
//    // error that only constant items are allowed, defensive programming etc...
//    let typed_decl = match typed_node.content {
//        TyAstNodeContent::Declaration(decl) => decl,
//        _ => {
//            return Err(
//                handler.emit_err(CompileError::ContractIdConstantNotAConstDecl {
//                    span: const_item_span,
//                }),
//            );
//        }
//    };
//    compiled_constants.insert(name, ResolvedDeclaration::Typed(typed_decl));
//
//    let mut ret = Module::new(ns_name, visibility, None);
//    ret.current_lexical_scope_mut().items.symbols = compiled_constants;
//    Ok(ret)
//}

/// Factory function for contracts
pub fn namespace_without_contract_id(
    package_name: Ident,
) -> Root {
    Root::new(package_name, None, false)
}

/// Factory function for contracts
pub fn namespace_with_contract_id(
    engines: &Engines,
    package_name: Ident,
    contract_id_value: String,
    experimental: crate::ExperimentalFlags,
) -> Result<Root, vec1::Vec1<CompileError>> {
    let root = Root::new(package_name, None, true);
    let handler = <_>::default();
    bind_contract_id_in_root_module(&handler, engines, contract_id_value, root, experimental)
	.map_err(|_| {
            let (errors, warnings) = handler.consume();
            assert!(warnings.is_empty());
	    
            // Invariant: `.value == None` => `!errors.is_empty()`.
            vec1::Vec1::try_from_vec(errors).unwrap()
	})
}

fn bind_contract_id_in_root_module(
    handler: &Handler,
    engines: &Engines,
    contract_id_value: String,
    root: Root,
    experimental: crate::ExperimentalFlags,
) -> Result<Root, ErrorEmitted> {
    // this for loop performs a miniature compilation of each const item in the config
    // FIXME(Centril): Stop parsing. Construct AST directly instead!
    // parser config
    let const_item = format!("pub const {CONTRACT_ID}: b256 = {contract_id_value};");
    let const_item_len = const_item.len();
    let input_arc = std::sync::Arc::from(const_item);
    let token_stream = lex(handler, &input_arc, 0, const_item_len, None).unwrap();
    let mut parser = Parser::new(handler, &token_stream);
    // perform the parse
    let const_item: ItemConst = parser.parse()?;
    let const_item_span = const_item.span();

    // perform the conversions from parser code to parse tree types
    let attributes = Default::default();
    // convert to const decl
    let const_decl_id = to_parsed_lang::item_const_to_constant_declaration(
        &mut to_parsed_lang::Context::new(crate::BuildTarget::EVM, experimental),
        handler,
        engines,
        const_item,
        attributes,
        true,
    )?;

    // Temporarily disallow non-literals. See https://github.com/FuelLabs/sway/issues/2647.
    let const_decl = engines.pe().get_constant(&const_decl_id);
    let has_literal = match &const_decl.value {
        Some(value) => {
            matches!(value.kind, ExpressionKind::Literal(_))
        }
        None => false,
    };

    if !has_literal {
        return Err(handler.emit_err(CompileError::ContractIdValueNotALiteral {
            span: const_item_span,
        }));
    }

    let ast_node = AstNode {
        content: AstNodeContent::Declaration(Declaration::ConstantDeclaration(const_decl_id)),
        span: const_item_span.clone(),
    };

    // This is pretty hacky but that's okay because of this code is being removed pretty soon
    // The root object 
    let mut namespace = Namespace::new(handler, engines, root, false)?;
    let type_check_ctx = TypeCheckContext::from_namespace(&mut namespace, engines, experimental);
    // Typecheck the const declaration. This will add the binding in the supplied namespace
    match TyAstNode::type_check(handler, type_check_ctx, &ast_node).unwrap().content {
        TyAstNodeContent::Declaration(_) => Ok(namespace.root()),
        _ => {
	    // TODO: Should this not be an ICE? If the typecheck fails then it's because our own
	    // hardcoded declaration is wrong.
            Err(
                handler.emit_err(CompileError::ContractIdConstantNotAConstDecl {
                    span: const_item_span,
                }),
            )
        },
    }
}
