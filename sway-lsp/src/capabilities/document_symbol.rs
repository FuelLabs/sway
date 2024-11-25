use crate::core::token::{get_range_from_span, SymbolKind};
use lsp_types::{self, DocumentSymbol, Url};
use sway_core::{
    language::ty::{
        TyAbiDecl, TyAstNodeContent, TyConstantDecl, TyDecl, TyEnumDecl, TyFunctionDecl,
        TyFunctionParameter, TyImplSelfOrTrait, TyProgram, TyStorageDecl, TyStructDecl,
        TyTraitDecl, TyTraitInterfaceItem, TyTraitItem, TyTraitType,
    },
    Engines, TypeArgument,
};
use sway_types::{Span, Spanned};

pub fn to_document_symbols<'a>(
    uri: &Url,
    ty_program: &'a TyProgram,
    engines: &Engines,
) -> Vec<DocumentSymbol> {
    let path = uri.to_file_path().unwrap();
    let source_id = engines.se().get_source_id(&path);
    let mut nodes: Vec<_> = ty_program.root.all_nodes
        .iter()
        .filter_map(|node| 
            // Iterare over the ast_nodes and filter_out any that don't originate from the file. We filter out carry forward Declaration nodes.
            matches!(node.content, TyAstNodeContent::Declaration(_) if node.span.source_id() == Some(&source_id))
            .then(|| &node.content)
            .and_then(|content| if let TyAstNodeContent::Declaration(n) = content { Some(n) } else { None })
        )
        .filter_map(|n| {
            match n {
                TyDecl::FunctionDecl(decl) => {
                    let fn_decl = engines.de().get_function(&decl.decl_id);
                    let range= get_range_from_span(&fn_decl.name.span());
                    let detail = Some(fn_decl_detail(&fn_decl.parameters, &fn_decl.return_type));
                    let children = collect_variables_from_func_decl(engines, &fn_decl);
                    let func_symbol = DocumentSymbolBuilder::new()
                        .name(fn_decl.name.span().str().to_string())
                        .kind(lsp_types::SymbolKind::FUNCTION)
                        .range(range)
                        .selection_range(range)
                        .detail(detail)
                        .children(children)
                        .build();
                    Some(func_symbol)
                }
                TyDecl::EnumDecl(decl) => {
                    let enum_decl = engines.de().get_enum(&decl.decl_id);
                    let span = enum_decl.call_path.suffix.span();
                    let range = get_range_from_span(&span);
                    let children = collect_enum_variants(&enum_decl);
                    let enum_symbol = DocumentSymbolBuilder::new()
                        .name(span.str().to_string())
                        .kind(lsp_types::SymbolKind::ENUM)
                        .range(range)
                        .selection_range(range)
                        .children(children)
                        .build();
                    Some(enum_symbol)
                }
                TyDecl::StructDecl(decl) => {
                    let struct_decl = engines.de().get_struct(&decl.decl_id);
                    let span = struct_decl.call_path.suffix.span();
                    let range = get_range_from_span(&span);
                    let children = collect_struct_fields(&struct_decl);
                    let struct_symbol = DocumentSymbolBuilder::new()
                        .name(span.str().to_string())
                        .kind(lsp_types::SymbolKind::STRUCT)
                        .range(range)
                        .selection_range(range)
                        .children(children)
                        .build();
                    Some(struct_symbol)
                }
                TyDecl::AbiDecl(decl) => {
                    let abi_decl = engines.de().get_abi(&decl.decl_id);
                    let decl_str = format!("{}", abi_decl.span().str());
                    let name = extract_header(&decl_str);
                    let range = get_range_from_span(&abi_decl.name.span());
                    let children = collect_fns_from_abi_decl(engines, &abi_decl);
                    let abi_symbol = DocumentSymbolBuilder::new()
                        .name(name)
                        .kind(lsp_types::SymbolKind::NAMESPACE)
                        .range(range)
                        .selection_range(range)
                        .children(children)
                        .build();
                    Some(abi_symbol)
                }
                TyDecl::TraitDecl(decl) => {
                    let trait_decl = engines.de().get_trait(&decl.decl_id);
                    let decl_str = format!("{}", trait_decl.span().str());
                    let name = extract_header(&decl_str);
                    let range = get_range_from_span(&trait_decl.name.span());
                    let children = collect_interface_surface(engines, &trait_decl.interface_surface);
                    let trait_symbol = DocumentSymbolBuilder::new()
                        .name(name)
                        .kind(lsp_types::SymbolKind::INTERFACE)
                        .range(range)
                        .selection_range(range)
                        .children(children)
                        .build();
                    Some(trait_symbol)
                }
                TyDecl::TraitTypeDecl(decl) => {
                    let trait_type_decl = engines.de().get_type(&decl.decl_id);
                    Some(build_trait_symbol(&trait_type_decl))
                }
                TyDecl::ImplSelfOrTrait(decl) => {
                    let impl_trait_decl = engines.de().get_impl_self_or_trait(&decl.decl_id);
                    let decl_str = format!("{}", impl_trait_decl.span().str());
                    let name = extract_header(&decl_str);
                    let range = get_range_from_span(&impl_trait_decl.trait_name.suffix.span());
                    let children = collect_ty_trait_items(engines, &impl_trait_decl.items);
                    let symbol = DocumentSymbolBuilder::new()
                        .name(name)
                        .kind(lsp_types::SymbolKind::NAMESPACE)
                        .range(range)
                        .selection_range(range)
                        .children(children)
                        .build();
                    Some(symbol)
                }
                TyDecl::ConstantDecl(decl) => {
                    let const_decl = engines.de().get_constant(&decl.decl_id);
                    Some(build_constant_symbol(&const_decl))
                }
                TyDecl::StorageDecl(decl) => {
                    let storage_decl = engines.de().get_storage(&decl.decl_id);
                    let span = storage_decl.storage_keyword.span();
                    let range = get_range_from_span(&span);
                    let children = collect_fields_from_storage(&storage_decl);
                    let storage_symbol = DocumentSymbolBuilder::new()
                        .name(span.str().to_string())
                        .kind(lsp_types::SymbolKind::STRUCT)
                        .range(range)
                        .selection_range(range)
                        .children(children)
                        .build();
                    Some(storage_symbol)
                }
                TyDecl::ConfigurableDecl(decl) => {
                    // Ideally we would show these as children of the configurable symbol
                    let configurable_decl = engines.de().get_configurable(&decl.decl_id);
                    let span = configurable_decl.call_path.suffix.span();
                    let range = get_range_from_span(&span);
                    let configurable_symbol = DocumentSymbolBuilder::new()
                        .name(span.str().to_string())
                        .kind(lsp_types::SymbolKind::STRUCT)
                        .detail(Some(format!("{}", configurable_decl.type_ascription.span.as_str())))
                        .range(range)
                        .selection_range(range)
                        .build();
                    Some(configurable_symbol)
                }
                _ => None
            }
        })
        .collect();

    // Sort by range start position
    nodes.sort_by_key(|node| node.range.start);
    nodes
}

fn build_constant_symbol(const_decl: &TyConstantDecl) -> DocumentSymbol {
    let span = const_decl.call_path.suffix.span();
    let range = get_range_from_span(&span);
    DocumentSymbolBuilder::new()
        .name(span.str().to_string())
        .kind(lsp_types::SymbolKind::CONSTANT)
        .detail(Some(format!(
            "{}",
            const_decl.type_ascription.span.as_str()
        )))
        .range(range)
        .selection_range(range)
        .build()
}

fn build_trait_symbol(trait_type_decl: &TyTraitType) -> DocumentSymbol {
    let span = trait_type_decl.name.span();
    let range = get_range_from_span(&span);
    DocumentSymbolBuilder::new()
        .name(span.str().to_string())
        .kind(lsp_types::SymbolKind::TYPE_PARAMETER)
        .range(range)
        .selection_range(range)
        .build()
}

fn collect_interface_surface(
    engines: &Engines,
    items: &[TyTraitInterfaceItem],
) -> Vec<DocumentSymbol> {
    items
        .iter()
        .map(|item| match item {
            TyTraitInterfaceItem::TraitFn(decl_ref) => {
                let fn_decl = engines.de().get_trait_fn(decl_ref);
                build_function_symbol(
                    &fn_decl.name.span(),
                    &fn_decl.parameters,
                    &fn_decl.return_type,
                )
            }
            TyTraitInterfaceItem::Constant(decl_ref) => {
                let const_decl = engines.de().get_constant(decl_ref);
                build_constant_symbol(&const_decl)
            }
            TyTraitInterfaceItem::Type(decl_ref) => {
                let trait_type_decl = engines.de().get_type(decl_ref);
                build_trait_symbol(&trait_type_decl)
            }
        })
        .collect()
}

fn collect_ty_trait_items(engines: &Engines, items: &[TyTraitItem]) -> Vec<DocumentSymbol> {
    items
        .iter()
        .filter_map(|item| match item {
            TyTraitItem::Fn(decl_ref) => Some(engines.de().get_function(decl_ref)),
            _ => None,
        })
        .map(|fn_decl| {
            let children = collect_variables_from_func_decl(engines, &fn_decl);
            let mut symbol = build_function_symbol(
                &fn_decl.name.span(),
                &fn_decl.parameters,
                &fn_decl.return_type,
            );
            symbol.children = Some(children);
            symbol
        })
        .collect()
}

fn collect_fields_from_storage(decl: &TyStorageDecl) -> Vec<DocumentSymbol> {
    decl.fields
        .iter()
        .map(|field| build_field_symbol(&field.name.span(), &field.type_argument))
        .collect()
}

fn build_field_symbol(span: &Span, type_argument: &TypeArgument) -> DocumentSymbol {
    let range = get_range_from_span(&span);
    DocumentSymbolBuilder::new()
        .name(span.clone().str().to_string())
        .detail(Some(format!("{}", type_argument.span.as_str())))
        .kind(lsp_types::SymbolKind::FIELD)
        .range(range)
        .selection_range(range)
        .build()
}

fn build_function_symbol(
    span: &Span,
    parameters: &[TyFunctionParameter],
    return_type: &TypeArgument,
) -> DocumentSymbol {
    let range = get_range_from_span(span);
    DocumentSymbolBuilder::new()
        .name(span.clone().str().to_string())
        .detail(Some(fn_decl_detail(parameters, return_type)))
        .kind(lsp_types::SymbolKind::FUNCTION)
        .range(range)
        .selection_range(range)
        .build()
}

fn collect_fns_from_abi_decl(engines: &Engines, decl: &TyAbiDecl) -> Vec<DocumentSymbol> {
    decl.interface_surface
        .iter()
        .filter_map(|item| match item {
            TyTraitInterfaceItem::TraitFn(decl_ref) => Some(engines.de().get_trait_fn(decl_ref)),
            _ => None,
        })
        .map(|trait_fn| {
            build_function_symbol(
                &trait_fn.name.span(),
                &trait_fn.parameters,
                &trait_fn.return_type,
            )
        })
        .collect()
}

fn collect_struct_fields(decl: &TyStructDecl) -> Vec<DocumentSymbol> {
    decl.fields
        .iter()
        .map(|field| build_field_symbol(&field.name.span(), &field.type_argument))
        .collect()
}

// Collect all enum variants
fn collect_enum_variants(decl: &TyEnumDecl) -> Vec<DocumentSymbol> {
    decl.variants
        .iter()
        .map(|variant| {
            let range = get_range_from_span(&variant.name.span());
            let symbol = DocumentSymbolBuilder::new()
                .name(variant.name.span().str().to_string())
                .kind(lsp_types::SymbolKind::ENUM_MEMBER)
                .range(range)
                .selection_range(range)
                .build();
            symbol
        })
        .collect()
}

// Collect all variables declared within the function body
fn collect_variables_from_func_decl(
    engines: &Engines,
    decl: &TyFunctionDecl,
) -> Vec<DocumentSymbol> {
    decl.body
        .contents
        .iter()
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
        .collect()
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

/// Extracts the header of a sway construct such as an `impl` block or `abi` declaration,
/// including any generic parameters, traits, or super traits, up to (but not including)
/// the opening `{` character. Trims any trailing whitespace.
///
/// If the `{` character is not found, the entire string is returned without trailing whitespace.
///
/// # Examples
///
/// ```
/// let impl_example = "impl<T> Setter<T> for FooBarData<T> {\n    fn set(self, new_value: T) -> Self {\n        FooBarData {\n            value: new_value,\n        }\n    }\n}";
/// let result = extract_header(impl_example);
/// assert_eq!(result, "impl<T> Setter<T> for FooBarData<T>");
///
/// let abi_example = "abi MyAbi : MySuperAbi {\n    fn bar();\n}";
/// let result = extract_header(abi_example);
/// assert_eq!(result, "abi MyAbi : MySuperAbi");
/// ```
fn extract_header(s: &str) -> &str {
    if let Some(pos) = s.find('{') {
        s[..pos].trim_end()
    } else {
        s.trim_end()
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
            range: lsp_types::Range::new(
                lsp_types::Position::new(0, 0),
                lsp_types::Position::new(0, 0),
            ),
            selection_range: lsp_types::Range::new(
                lsp_types::Position::new(0, 0),
                lsp_types::Position::new(0, 0),
            ),
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
        #[allow(warnings)]
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
