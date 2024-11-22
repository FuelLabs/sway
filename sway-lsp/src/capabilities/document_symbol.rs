use crate::core::{
    token::{get_range_from_span, AstToken, SymbolKind, Token, TokenIdent, TypedAstToken},
    token_map::{self, TokenMap},
};
use dashmap::mapref::multiple::RefMulti;
use lsp_types::{self, DocumentSymbol, Url};
use sway_core::{language::ty::{TyAstNodeContent, TyDecl}, Engines};
use sway_types::Spanned;

// #[derive(Debug)]
struct SymbolNode {
    symbol: DocumentSymbol,
    range_start: u32,
    ident: TokenIdent,
    token: Token,
    // t: RefMulti<'a, TokenIdent, Token>,
}

pub fn to_document_symbols<'a>(
    uri: &Url,
    token_map: &'a TokenMap,
    engines: &Engines,
) -> Vec<DocumentSymbol> {
    let tokens_for_file = token_map.tokens_for_file(uri);
    let mut nodes = tokens_for_file
        .map(|entry| {
            let (ident, token) = entry.pair();
            create_symbol_node(ident, token)
        })
        .collect::<Vec<SymbolNode>>();
    nodes.sort_by_key(|node| node.range_start);
    build_symbol_hierarchy(nodes, engines)
}

fn build_symbol_hierarchy(nodes: Vec<SymbolNode>, engines: &Engines) -> Vec<DocumentSymbol> {
    let mut result = Vec::new();
    let mut current_struct: Option<DocumentSymbol> = None;
    let mut struct_fields = Vec::new();

    let mut current_enum: Option<DocumentSymbol> = None;
    let mut enum_variants = Vec::new();

    for node in nodes {
        let is_declaration = match node.token.typed {
            Some(TypedAstToken::TypedDeclaration(_)) => true,
            None => match node.token.parsed {
                AstToken::Declaration(_) => true,
                _ => false,
            },
            _ => false,
        };
        match node.symbol.kind {
            lsp_types::SymbolKind::STRUCT => {
                if is_declaration {
                    // Push previous struct if exists
                    if let Some(mut s) = current_struct.take() {
                        if !struct_fields.is_empty() {
                            s.children = Some(struct_fields);
                            struct_fields = Vec::new();
                        }
                        result.push(s);
                    }
                    current_struct = Some(node.symbol);
                }
            }
            lsp_types::SymbolKind::FIELD => {
                // Only collect struct field members if they belong to the struct declaration
                if let Some(decl_ident) = node.token.declared_token_ident(engines) {
                    if node.ident.range == decl_ident.range {
                        if current_struct.is_some() {
                            struct_fields.push(node.symbol);
                        }
                    }
                }
            }
            lsp_types::SymbolKind::ENUM => {
                if is_declaration {
                    // Push previous struct if exists
                    if let Some(mut s) = current_enum.take() {
                        if !enum_variants.is_empty() {
                            s.children = Some(enum_variants);
                            enum_variants = Vec::new();
                        }
                        result.push(s);
                    }
                    current_enum = Some(node.symbol);
                }
            }
            lsp_types::SymbolKind::ENUM_MEMBER => {
                // Only collect enum members if they belong to the enum declaration, we expect None in this case
                if node.token.declared_token_ident(engines).is_none() {
                    if current_enum.is_some() {
                        enum_variants.push(node.symbol);
                    }
                }
            }
            lsp_types::SymbolKind::FUNCTION => {
                if let Some(typed) = node.token.typed {
                    if let TypedAstToken::TypedFunctionDeclaration(fn_decl) = typed {
                        let mut variables = Vec::new();
                        for node in fn_decl.body.contents {
                            if let TyAstNodeContent::Declaration(decl) = node.content {
                                if let TyDecl::VariableDecl(var_decl) = decl {
                                    let range = get_range_from_span(&var_decl.name.span());
                                    let type_name = format!("{}", engines.help_out(var_decl.type_ascription.type_id));
                                    let detail = if type_name.is_empty() {
                                        None
                                    } else {
                                        Some(type_name)
                                    };
                                    let symbol = DocumentSymbol {
                                        name: var_decl.name.span().str().to_string(),
                                        detail: detail,
                                        kind: lsp_types::SymbolKind::VARIABLE,
                                        tags: None,
                                        range: range,
                                        selection_range: range,
                                        children: None,
                                        deprecated: None,
                                    };
                                    variables.push(symbol);
                                }
                            }
                        }
                        let mut fn_symbol = node.symbol.clone();
                        if !variables.is_empty() {
                            fn_symbol.children = Some(variables);
                        }
                        result.push(fn_symbol);
                    }
                }
            }
            _ => {
                if matches!(node.symbol.kind, lsp_types::SymbolKind::FUNCTION) {
                    // For everything else (including functions), just add to results in order
                    result.push(node.symbol);
                }
            }
            
        }
    }

    // Handle last struct
    if let Some(mut s) = current_struct {
        if !struct_fields.is_empty() {
            s.children = Some(struct_fields);
        }
        result.push(s);
    }

    // Handle last enum
    if let Some(mut s) = current_enum {
        if !enum_variants.is_empty() {
            s.children = Some(enum_variants);
        }
        result.push(s);
    }

    result
}

/// Given a `token::SymbolKind`, return the `lsp_types::SymbolKind` that corresponds to it.
fn symbol_kind(symbol_kind: &SymbolKind) -> lsp_types::SymbolKind {
    match symbol_kind {
        SymbolKind::Field => lsp_types::SymbolKind::FIELD,
        SymbolKind::BuiltinType | SymbolKind::TypeParameter => {
            lsp_types::SymbolKind::TYPE_PARAMETER
        }
        SymbolKind::Function | SymbolKind::Intrinsic => lsp_types::SymbolKind::FUNCTION,
        SymbolKind::DeriveHelper => lsp_types::SymbolKind::KEY,
        SymbolKind::Const => lsp_types::SymbolKind::CONSTANT,
        SymbolKind::Struct => lsp_types::SymbolKind::STRUCT,
        SymbolKind::Trait => lsp_types::SymbolKind::INTERFACE,
        SymbolKind::Module => lsp_types::SymbolKind::MODULE,
        SymbolKind::Enum => lsp_types::SymbolKind::ENUM,
        SymbolKind::Variant => lsp_types::SymbolKind::ENUM_MEMBER,
        SymbolKind::BoolLiteral => lsp_types::SymbolKind::BOOLEAN,
        SymbolKind::StringLiteral => lsp_types::SymbolKind::STRING,
        SymbolKind::NumericLiteral => lsp_types::SymbolKind::NUMBER,
        SymbolKind::ValueParam
        | SymbolKind::ByteLiteral
        | SymbolKind::Variable
        | SymbolKind::TypeAlias
        | SymbolKind::TraitType
        | SymbolKind::Keyword
        | SymbolKind::SelfKeyword
        | SymbolKind::SelfTypeKeyword
        | SymbolKind::ProgramTypeKeyword
        | SymbolKind::Unknown => lsp_types::SymbolKind::VARIABLE,
    }
}

#[allow(warnings)]
// TODO: the "deprecated: None" field is deprecated according to this library
fn create_symbol_node<'a>(ident: &'a TokenIdent, token: &'a Token) -> SymbolNode {
    let kind = symbol_kind(&token.kind);

    let detail = match &token.typed {
        Some(TypedAstToken::TypedStructField(field)) => {
            // show the type of the field
            Some(format!("{}", field.type_argument.span.as_str()))
        }
        Some(TypedAstToken::TypedFunctionDeclaration(fn_decl)) => {
            // For functions, show their signature
            let params = fn_decl
                .parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, p.type_argument.span.as_str()))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = fn_decl.return_type.span.as_str();
            Some(format!("fn({}) -> {}", params, return_type))
        }
        _ => None,
    };

    SymbolNode {
        symbol: DocumentSymbol {
            name: ident.name.to_string(),
            detail,
            kind,
            tags: None,
            range: ident.range,
            selection_range: ident.range,
            children: None,
            deprecated: None,
        },
        range_start: ident.range.start.line,
        ident: ident.clone(),
        token: token.clone(),
    }
}
