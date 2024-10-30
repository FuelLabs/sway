use crate::core::token::{SymbolKind, Token, TokenIdent, TypedAstToken};
use dashmap::mapref::multiple::RefMulti;
use lsp_types::{self, DocumentSymbol};

#[derive(Debug)]
struct SymbolNode {
    symbol: DocumentSymbol,
    range_start: u32,
}

pub fn to_document_symbols<'a, I>(tokens: I) -> Vec<DocumentSymbol>
where
    I: Iterator<Item = RefMulti<'a, TokenIdent, Token>>,
{
    let mut nodes = tokens
        .map(|entry| {
            let (ident, token) = entry.pair();
            create_symbol_node(ident, token)
        })
        .collect::<Vec<SymbolNode>>();

    nodes.sort_by_key(|node| node.range_start);
    build_symbol_hierarchy(nodes)
}

fn build_symbol_hierarchy(nodes: Vec<SymbolNode>) -> Vec<DocumentSymbol> {
    let mut result = Vec::new();
    let mut current_struct: Option<DocumentSymbol> = None;
    let mut struct_children = Vec::new();

    for node in nodes {
        match node.symbol.kind {
            lsp_types::SymbolKind::STRUCT => {
                // Push previous struct if exists
                if let Some(mut s) = current_struct.take() {
                    if !struct_children.is_empty() {
                        s.children = Some(struct_children);
                        struct_children = Vec::new();
                    }
                    result.push(s);
                }
                current_struct = Some(node.symbol);
            }
            lsp_types::SymbolKind::FIELD | lsp_types::SymbolKind::FUNCTION
                if current_struct.is_some() =>
            {
                struct_children.push(node.symbol);
            }
            _ => {
                // Top-level items
                if current_struct.is_none() {
                    result.push(node.symbol);
                }
            }
        }
    }

    // Handle last struct
    if let Some(mut s) = current_struct {
        if !struct_children.is_empty() {
            s.children = Some(struct_children);
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
fn create_symbol_node(ident: &TokenIdent, token: &Token) -> SymbolNode {
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
    }
}
