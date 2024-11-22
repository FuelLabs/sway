use crate::core::{
    token::{get_range_from_span, SymbolKind, Token, TokenIdent, TypedAstToken},
    token_map::{self, TokenMap},
};
use dashmap::mapref::multiple::RefMulti;
use lsp_types::{self, DocumentSymbol, Url};
use sway_core::{language::ty::{TyAstNodeContent, TyDecl, TyFunctionDecl, TyFunctionParameter, TyTraitInterfaceItem, TyTraitItem}, Engines, TypeArgument};
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
    // let decl_tokens_for_file: Vec<_> = token_map.tokens_for_file(uri).filter_map(|t| {
    //     match t.typed {
    //         Some(TypedAstToken::TypedDeclaration(_)) 
    //         | Some(TypedAstToken::TypedFunctionDeclaration(_)) 
    //         | Some(TypedAstToken::TypedConstantDeclaration(_))
    //         | Some(TypedAstToken::TypedConfigurableDeclaration(_))
    //         | Some(TypedAstToken::TypedTraitTypeDeclaration(_)) => {
    //             Some(t.typed.clone())
    //         }
    //         _ => None,
    //     }
    // }).collect();

    // decl_tokens_for_file


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

    for node in nodes {
        match node.symbol.kind {
            lsp_types::SymbolKind::INTERFACE => {
                let methods: Vec<_> = match node.token.typed {
                    Some(TypedAstToken::TypedDeclaration(TyDecl::TraitDecl(trait_decl))) => {
                        engines.de()
                            .get_trait(&trait_decl.decl_id)
                            .items
                            .iter()
                            .filter_map(|trait_item| {
                                if let TyTraitItem::Fn(decl_ref) = trait_item {
                                    Some(engines.de().get_function(decl_ref))
                                } else {
                                    None
                                }
                            })
                            .map(|fn_decl| {
                                let range = get_range_from_span(&fn_decl.name.span());
                                DocumentSymbolBuilder::new()
                                    .name(fn_decl.name.span().str().to_string())
                                    .detail(Some(fn_decl_detail(&fn_decl.parameters, &fn_decl.return_type)))
                                    .kind(lsp_types::SymbolKind::FUNCTION)
                                    .range(range)
                                    .selection_range(range)
                                    .build()
                            })
                            .collect()
                    }
                    Some(TypedAstToken::TypedDeclaration(TyDecl::AbiDecl(abi_decl))) => {
                        engines.de()
                            .get_abi(&abi_decl.decl_id)
                            .interface_surface
                            .iter()
                            .filter_map(|trait_item| {
                                if let TyTraitInterfaceItem::TraitFn(decl_ref) = trait_item {
                                    Some(engines.de().get_trait_fn(decl_ref))
                                } else {
                                    None
                                }
                            })
                            .map(|trait_fn| {
                                let range = get_range_from_span(&trait_fn.name.span());
                                DocumentSymbolBuilder::new()
                                    .name(trait_fn.name.span().str().to_string())
                                    .detail(Some(fn_decl_detail(&trait_fn.parameters, &trait_fn.return_type)))
                                    .kind(lsp_types::SymbolKind::FUNCTION)
                                    .range(range)
                                    .selection_range(range)
                                    .build()
                            })
                            .collect()
                    }
                    _ => vec![],
                };
                let mut trait_symbol = node.symbol.clone();
                if !methods.is_empty() {
                    trait_symbol.children = Some(methods);
                }
                result.push(trait_symbol);
            }
            lsp_types::SymbolKind::STRUCT => {
                if let Some(TypedAstToken::TypedDeclaration(TyDecl::StructDecl(struct_decl))) = node.token.typed {
                    let fields: Vec<_> = engines.de()
                        .get_struct(&struct_decl.decl_id)
                        .fields
                        .iter()
                        .map(|field| {
                            let range = get_range_from_span(&field.name.span());
                            DocumentSymbolBuilder::new()
                                .name(field.name.span().str().to_string())
                                .detail(Some(format!("{}", field.type_argument.span.as_str())))
                                .kind(lsp_types::SymbolKind::FIELD)
                                .range(range)
                                .selection_range(range)
                                .build()
                        })
                        .collect();
                    let mut struct_symbol = node.symbol.clone();
                    if !fields.is_empty() {
                        struct_symbol.children = Some(fields);
                    }
                    result.push(struct_symbol);
                }
            }
            lsp_types::SymbolKind::ENUM => {
                if let Some(TypedAstToken::TypedDeclaration(TyDecl::EnumDecl(enum_decl))) = node.token.typed {
                    let variants: Vec<_> = engines.de()
                        .get_enum(&enum_decl.decl_id)
                        .variants
                        .iter()
                        .map(|variant| {
                            let range = get_range_from_span(&variant.name.span());
                            DocumentSymbolBuilder::new()
                                .name(variant.name.span().str().to_string())
                                .kind(lsp_types::SymbolKind::ENUM_MEMBER)
                                .range(range)
                                .selection_range(range)
                                .build()
                        })
                        .collect();
                    let mut enum_symbol = node.symbol.clone();
                    if !variants.is_empty() {
                        enum_symbol.children = Some(variants);
                    }
                    result.push(enum_symbol);
                }
            }
            lsp_types::SymbolKind::FUNCTION => {
                if let Some(TypedAstToken::TypedFunctionDeclaration(fn_decl)) = node.token.typed {
                    // Collect all variables declared within the function body
                    let variables: Vec<_> = fn_decl.body.contents.iter()
                        .filter_map(|node| {
                            if let TyAstNodeContent::Declaration(TyDecl::VariableDecl(var_decl)) = &node.content {
                                let range = get_range_from_span(&var_decl.name.span());
                                let type_name = format!("{}", engines.help_out(var_decl.type_ascription.type_id));
                                let symbol = DocumentSymbolBuilder::new()
                                    .name(var_decl.name.span().str().to_string())
                                    .kind(lsp_types::SymbolKind::VARIABLE)
                                    .range(range)
                                    .selection_range(range)
                                    .detail((!type_name.is_empty()).then_some(type_name))
                                    .build();
                                Some(symbol)
                            } else {
                                None
                            }
                        })
                        .collect();
                    let mut fn_symbol = node.symbol.clone();
                    if !variables.is_empty() {
                        // Add the variables to the function symbol
                        fn_symbol.children = Some(variables);
                    }
                    result.push(fn_symbol);
                }
            }
            _ => {
                
            }
        }
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

// Generate the signature for functions
fn fn_decl_detail(parameters: &[TyFunctionParameter], return_type: &TypeArgument) -> String {
    let params = parameters
        .iter()
        .map(|p| format!("{}: {}", p.name, p.type_argument.span.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    let return_type = return_type.span.as_str();
    format!("fn({}) -> {}", params, return_type)
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
            Some(fn_decl_detail(&fn_decl.parameters, &fn_decl.return_type))
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

/// Builder for creating [`DocumentSymbol`] instances with method chaining.
/// Initializes with empty name, NULL kind, and zero position ranges. 
pub struct DocumentSymbolBuilder {
    name: String,
    detail: Option<String>,
    kind: lsp_types::SymbolKind,
    tags: Option<Vec<lsp_types::SymbolTag>>,
    range: lsp_types::Range,
    selection_range: lsp_types::Range,
    children: Option<Vec<DocumentSymbol>>,
    deprecated: Option<bool>,
}

impl DocumentSymbolBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            kind: lsp_types::SymbolKind::NULL,
            range: lsp_types::Range::new(lsp_types::Position::new(0, 0), lsp_types::Position::new(0, 0)),
            selection_range: lsp_types::Range::new(lsp_types::Position::new(0, 0), lsp_types::Position::new(0, 0)),
            detail: None,
            tags: None,
            children: None,
            deprecated: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn kind(mut self, kind: lsp_types::SymbolKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn range(mut self, range: lsp_types::Range) -> Self {
        self.range = range;
        self
    }

    pub fn selection_range(mut self, range: lsp_types::Range) -> Self {
        self.selection_range = range;
        self
    }

    pub fn detail(mut self, detail: Option<String>) -> Self {
        self.detail = detail;
        self
    }

    pub fn tags(mut self, tags: Vec<lsp_types::SymbolTag>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn children(mut self, children: Vec<DocumentSymbol>) -> Self {
        self.children = Some(children);
        self
    }

    pub fn build(self) -> DocumentSymbol {
        DocumentSymbol {
            name: self.name,
            detail: self.detail,
            kind: self.kind,
            tags: self.tags,
            range: self.range,
            selection_range: self.selection_range,
            children: self.children,
            deprecated: self.deprecated,
        }
    }
}