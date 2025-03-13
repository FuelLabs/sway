use sway_ast::ItemConst;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_parse::{lex, Parser};
use sway_types::{constants::CONTRACT_ID, ProgramId, Spanned};

use crate::{
    build_config::DbgGeneration, language::{
        parsed::{AstNode, AstNodeContent, Declaration, ExpressionKind},
        ty::{TyAstNode, TyAstNodeContent},
    }, semantic_analysis::{
        namespace::Root, symbol_collection_context::SymbolCollectionContext, TypeCheckContext,
    }, transform::to_parsed_lang, Engines, Ident, Namespace
};

/// Factory function for contracts
pub fn namespace_with_contract_id(
    engines: &Engines,
    package_name: Ident,
    program_id: ProgramId,
    contract_id_value: String,
    experimental: crate::ExperimentalFeatures,
    dbg_generation: DbgGeneration,
) -> Result<Root, vec1::Vec1<CompileError>> {
    let root = Root::new(package_name, None, program_id, true);
    let handler = <_>::default();
    bind_contract_id_in_root_module(&handler, engines, contract_id_value, root, experimental, dbg_generation)
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
    experimental: crate::ExperimentalFeatures,
    dbg_generation: DbgGeneration,
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
        &mut to_parsed_lang::Context::new(crate::BuildTarget::EVM, dbg_generation, experimental),
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
    let mut symbol_ctx = SymbolCollectionContext::new(namespace.clone());
    let type_check_ctx =
        TypeCheckContext::from_namespace(&mut namespace, &mut symbol_ctx, engines, experimental);
    // Typecheck the const declaration. This will add the binding in the supplied namespace
    let type_checked = TyAstNode::type_check(handler, type_check_ctx, &ast_node).unwrap();
    if let TyAstNodeContent::Declaration(_) = type_checked.content {
        Ok(namespace.root())
    } else {
        Err(handler.emit_err(CompileError::Internal(
            "Contract ID declaration did not typecheck to a declaration, which should be impossible",
            const_item_span,
        )))
    }
}
