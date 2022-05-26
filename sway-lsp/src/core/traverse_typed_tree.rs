use std::{
    collections::BTreeMap,
};
use sway_types::{
    ident::Ident,
    span::Span,
};
use sway_core::semantic_analysis::ast_node::{
    {TypedAstNode, TypedAstNodeContent, TypedDeclaration, TypedFunctionDeclaration, TypedFunctionParameter, TypedStructField, 
    TypedEnumVariant, TypedTraitFn, TypedStorageField, TypeCheckedStorageReassignDescriptor, ReassignmentLhs},
    expression::{
        typed_expression::TypedExpression,
        typed_expression_variant::TypedExpressionVariant,
    },
    while_loop::TypedWhileLoop,
};
use sway_core::type_engine::TypeId;
use sway_core::parse_tree::literal::Literal;
use tower_lsp::lsp_types::{Position, Range, SymbolKind};


// #[derive(Debug, Clone)]
// pub struct TypedIdent {
//     pub name: Ident,
//     pub r#type: TypeId,
// }

// pub struct Token {
//     TypedIdent(TypedIdent),
//     TypedAstNodeContent(TypedAstNodeContent),
// }

#[derive(Debug, Clone)]
enum TokenType {
    TypedDeclaration(TypedDeclaration),
    TypedExpression(TypedExpression),

    TypedFunctionDeclaration(TypedFunctionDeclaration),
    TypedFunctionParameter(TypedFunctionParameter),
    TypedStructField(TypedStructField),
    TypedEnumVariant(TypedEnumVariant),
    TypedTraitFn(TypedTraitFn),
    TypedStorageField(TypedStorageField),
    TypeCheckedStorageReassignDescriptor(TypeCheckedStorageReassignDescriptor),
    ReassignmentLhs(ReassignmentLhs),
}

pub fn traverse_node(node: &TypedAstNode, tokens: &mut BTreeMap<Ident, TypedAstNodeContent>) {
    match &node.content {
        TypedAstNodeContent::ReturnStatement(return_statement) => handle_expression(&return_statement.expr, tokens),
        TypedAstNodeContent::Declaration(declaration) => handle_declaration(declaration, tokens),
        TypedAstNodeContent::Expression(expression) => handle_expression(expression, tokens),
        TypedAstNodeContent::ImplicitReturnExpression(expression) => handle_expression(expression, tokens),
        TypedAstNodeContent::WhileLoop(while_loop) => handle_while_loop(while_loop, tokens),
        TypedAstNodeContent::SideEffect => (),
    };
}

fn handle_declaration(declaration: &TypedDeclaration, tokens: &mut BTreeMap<Ident, TypedAstNodeContent>) {
    //eprintln!("DECLARATION = {:#?}", declaration);
    match declaration {
        TypedDeclaration::VariableDeclaration(variable) => {
            tokens.insert(variable.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            handle_expression(&variable.body, tokens);
        }
        TypedDeclaration::ConstantDeclaration(const_decl) => {
            tokens.insert(const_decl.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            handle_expression(&const_decl.value, tokens);
        }
        TypedDeclaration::FunctionDeclaration(func) => {
            tokens.insert(func.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            for node in &func.body.contents {
                traverse_node(node, tokens);
            }
        }
        TypedDeclaration::TraitDeclaration(trait_decl) => {
            tokens.insert(trait_decl.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
        }
        TypedDeclaration::StructDeclaration(struct_dec) => {
            tokens.insert(struct_dec.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            for field in &struct_dec.fields {
                tokens.insert(field.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            }
        }
        TypedDeclaration::EnumDeclaration(enum_decl) => {
            tokens.insert(enum_decl.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            for variant in &enum_decl.variants {
                tokens.insert(variant.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            }
        }  
        TypedDeclaration::Reassignment(reassignment) => {
            handle_expression(&reassignment.rhs, tokens);
            for lhs in &reassignment.lhs {
                tokens.insert(lhs.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            }
        }  
        TypedDeclaration::ImplTrait{trait_name, methods,..} => {
            for ident in &trait_name.prefixes {
                tokens.insert(ident.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            }
            tokens.insert(trait_name.suffix.clone(), TypedAstNodeContent::Declaration(declaration.clone()));

            for method in methods {
                tokens.insert(method.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
                for node in &method.body.contents {
                    traverse_node(node, tokens);
                }
                for paramater in &method.parameters {
                    tokens.insert(paramater.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
                }
            }
        }  
        TypedDeclaration::AbiDeclaration(abi_decl) => {
            tokens.insert(abi_decl.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            for trait_fn in &abi_decl.interface_surface {
                tokens.insert(trait_fn.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            }
        }  
        TypedDeclaration::GenericTypeForFunctionScope{name} => {
            tokens.insert(name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
        }  
        TypedDeclaration::ErrorRecovery => {}  
        TypedDeclaration::StorageDeclaration(storage_decl) => {
            for field in &storage_decl.fields {
                tokens.insert(field.name.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            }
        }  
        TypedDeclaration::StorageReassignment(storage_reassignment) => {
            for ident in &storage_reassignment.names() {
                tokens.insert(ident.clone(), TypedAstNodeContent::Declaration(declaration.clone()));
            }
            handle_expression(&storage_reassignment.rhs, tokens);
        }        
    }
}

fn handle_expression(expression: &TypedExpression, tokens: &mut BTreeMap<Ident, TypedAstNodeContent>) {
    match &expression.expression {
        TypedExpressionVariant::Literal{..} => {}
        TypedExpressionVariant::FunctionApplication{name, contract_call_params, arguments,function_body, ..} => {
            for ident in &name.prefixes {
                tokens.insert(ident.clone(), TypedAstNodeContent::Expression(expression.clone()));
            }
            tokens.insert(name.suffix.clone(), TypedAstNodeContent::Expression(expression.clone()));

            for exp in contract_call_params.values() {
                handle_expression(&exp, tokens);
            }

            for (ident, exp) in arguments {
                tokens.insert(ident.clone(), TypedAstNodeContent::Expression(exp.clone()));
            }

            for node in &function_body.contents {
                traverse_node(node, tokens);
            }
        }
        TypedExpressionVariant::LazyOperator{lhs, rhs,..} => {
            handle_expression(lhs, tokens);
            handle_expression(rhs, tokens);
        }
        TypedExpressionVariant::VariableExpression{ ref name } => {
            tokens.insert(name.clone(), TypedAstNodeContent::Expression(expression.clone()));
        }
        TypedExpressionVariant::Tuple{fields} => {
            for exp in fields {
                handle_expression(&exp, tokens);
            }   
        }
        TypedExpressionVariant::Array{ contents } => {
            for exp in contents {
                handle_expression(&exp, tokens);
            }
        }
        TypedExpressionVariant::ArrayIndex{prefix, index} => {
            handle_expression(prefix, tokens);
            handle_expression(index, tokens);
        } 
        TypedExpressionVariant::StructExpression{ ref struct_name, ref fields } => { 
            tokens.insert(struct_name.clone(), TypedAstNodeContent::Expression(expression.clone()));
            for field in fields { 
                tokens.insert(field.name.clone(), TypedAstNodeContent::Expression(field.value.clone()));
            }
        }
        TypedExpressionVariant::CodeBlock(code_block) => {
            for node in &code_block.contents {
                traverse_node(node, tokens);
            }
        }
        TypedExpressionVariant::FunctionParameter{..} => {}
        TypedExpressionVariant::IfExp{condition, then, r#else} => {
            handle_expression(condition, tokens);
            handle_expression(then, tokens);
            if let Some(r#else) = r#else {
                handle_expression(r#else, tokens);
            }
        }
        TypedExpressionVariant::AsmExpression{..} => {}
        TypedExpressionVariant::StructFieldAccess{prefix, field_to_access, ..} => {
            handle_expression(prefix, tokens);
            tokens.insert(field_to_access.name.clone(), TypedAstNodeContent::Expression(expression.clone()));
        }
        TypedExpressionVariant::IfLet{expr, variant, variable_to_assign, then, r#else, ..} => {
            handle_expression(&expr, tokens);
            tokens.insert(variant.name.clone(), TypedAstNodeContent::Expression(expression.clone()));
            tokens.insert(variable_to_assign.clone(), TypedAstNodeContent::Expression(expression.clone()));
            for node in &then.contents {
                traverse_node(node, tokens);
            }
            if let Some(r#else) = r#else {
                handle_expression(r#else, tokens);
            }
        }
        TypedExpressionVariant::TupleElemAccess{prefix, ..} => {
            handle_expression(prefix, tokens);
        }
        TypedExpressionVariant::EnumInstantiation{..} => {}
        TypedExpressionVariant::AbiCast{abi_name, address, ..} => {
            for ident in &abi_name.prefixes {
                tokens.insert(ident.clone(), TypedAstNodeContent::Expression(expression.clone()));
            }
            tokens.insert(abi_name.suffix.clone(), TypedAstNodeContent::Expression(expression.clone()));
            handle_expression(address, tokens);
        }
        TypedExpressionVariant::StorageAccess(storage_access) => {
            for field in &storage_access.fields {
                tokens.insert(field.name.clone(), TypedAstNodeContent::Expression(expression.clone()));
            }
        }
        TypedExpressionVariant::TypeProperty{..} => {}
        TypedExpressionVariant::SizeOfValue{expr} => {
            handle_expression(expr, tokens);
        }
        TypedExpressionVariant::AbiName{..} => {}
    }
}

fn handle_while_loop(while_loop: &TypedWhileLoop, tokens: &mut BTreeMap<Ident, TypedAstNodeContent>) {
    handle_expression(&while_loop.condition, tokens);
    for node in &while_loop.body.contents {
        traverse_node(node, tokens);
    }
}

pub fn type_id(typed_ast_node: &TypedAstNodeContent) -> Option<TypeId> {
    eprintln!("typed_ast_node = {:#?}", typed_ast_node);

    match typed_ast_node {
        TypedAstNodeContent::Declaration(dec) => {
            match dec {
                TypedDeclaration::VariableDeclaration(var_decl) => {
                    Some(var_decl.type_ascription)
                },
                TypedDeclaration::ConstantDeclaration(const_decl) => {
                    Some(const_decl.value.return_type)
                },
                _ => None,
            }
        }
        TypedAstNodeContent::Expression(exp) => {
            Some(exp.return_type)
        }
        _ => None,
    }
}

fn to_symbol_kind(typed_ast_node: &TypedAstNodeContent) -> SymbolKind {
    match typed_ast_node {
        TypedAstNodeContent::Declaration(dec) => {
            match dec {
                TypedDeclaration::VariableDeclaration(_) => SymbolKind::VARIABLE,
                TypedDeclaration::ConstantDeclaration(_) => SymbolKind::CONSTANT,
                TypedDeclaration::FunctionDeclaration(_) => SymbolKind::FUNCTION,
                TypedDeclaration::StructDeclaration(_) => SymbolKind::STRUCT,
                TypedDeclaration::EnumDeclaration(_) => SymbolKind::ENUM,
                TypedDeclaration::Reassignment(_) => SymbolKind::OPERATOR,
                TypedDeclaration::ImplTrait{..} => SymbolKind::INTERFACE,
                TypedDeclaration::AbiDeclaration(_) => SymbolKind::INTERFACE,
                TypedDeclaration::GenericTypeForFunctionScope{..} => SymbolKind::TYPE_PARAMETER,
                // currently we return `variable` type as default
                _ => SymbolKind::VARIABLE,
            }
        }
        TypedAstNodeContent::Expression(exp) => {
            match &exp.expression {
                TypedExpressionVariant::Literal(lit) => {
                    match lit {
                        Literal::String(_) => SymbolKind::STRING,
                        Literal::Boolean(_) => SymbolKind::BOOLEAN,
                        _ => SymbolKind::NUMBER,
                    }
                }
                TypedExpressionVariant::FunctionApplication{..} => SymbolKind::FUNCTION,
                TypedExpressionVariant::VariableExpression{..} => SymbolKind::VARIABLE,
                TypedExpressionVariant::Array{..} => SymbolKind::ARRAY,
                TypedExpressionVariant::StructExpression{..} => SymbolKind::STRUCT,
                TypedExpressionVariant::StructFieldAccess{..} => SymbolKind::FIELD,
                // currently we return `variable` type as default
                _ => SymbolKind::VARIABLE,
            }
        }
        // currently we return `variable` type as default
        _ => SymbolKind::VARIABLE,
    }
}

pub fn ident_at_position<'a>(cursor_position: Position, tokens: &'a BTreeMap<Ident, TypedAstNodeContent>) -> Option<&'a Ident> {
    for ident in tokens.keys() {
        let range = get_range_from_span(ident.span());
        if cursor_position >= range.start && cursor_position <= range.end {
            return Some(ident);
        }    
    }
    None
}

fn get_range_from_span(span: &Span) -> Range {
    let start = span.start_pos().line_col();
    let end = span.end_pos().line_col();

    let start_line = start.0 as u32 - 1;
    let start_character = start.1 as u32 - 1;

    let end_line = end.0 as u32 - 1;
    let end_character = end.1 as u32 - 1;

    Range {
        start: Position::new(start_line, start_character),
        end: Position::new(end_line, end_character),
    }
}

// DEBUG 
pub fn debug_print_ident(ident: &Ident, token: &TypedAstNodeContent) {
	let pos = ident.span().start_pos().line_col();
	let line_num = pos.0 as u32;	
    eprintln!("line num = {:?} | name: = {:?} | ast_node_type = {:?} | type_id = {:?}", 
        line_num,
        ident.as_str(),
        ast_node_type(&token),
        type_id(&token),
    );
}

fn ast_node_type(token: &TypedAstNodeContent) -> String {
    match &token {
        TypedAstNodeContent::Declaration(dec) => {
            dec.friendly_name().to_string()
        }
        TypedAstNodeContent::Expression(exp) => {
            exp.expression.pretty_print()
        }
        _ => "".to_string()
    }
}
