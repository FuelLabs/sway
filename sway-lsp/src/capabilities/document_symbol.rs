use crate::core::{token::get_range_from_span, token_map::TokenMap};
use lsp_types::{self, DocumentSymbol, Url};
use std::path::PathBuf;
use sway_core::{
    language::ty::{
        TyAbiDecl, TyAstNodeContent, TyConstantDecl, TyDecl, TyEnumDecl, TyFunctionDecl,
        TyFunctionParameter, TyIncludeStatement, TyProgram, TySideEffect, TyStorageDecl,
        TyStructDecl, TyTraitInterfaceItem, TyTraitItem, TyTraitType,
    },
    Engines, GenericArgument,
};
use sway_types::{Span, Spanned};

/// Generates a hierarchical document symbol tree for LSP code outline/navigation.
/// Processes declarations (functions, structs, enums, etc.) into nested symbols,
/// preserving parent-child relationships like functions with their variables,
/// structs with their fields, and traits with their methods.
pub fn to_document_symbols(
    uri: &Url,
    path: &PathBuf,
    ty_program: &TyProgram,
    engines: &Engines,
    token_map: &TokenMap,
) -> Vec<DocumentSymbol> {
    let source_id = engines.se().get_source_id(path);
    // Find if there is a configurable symbol in the token map that belongs to the current file
    // We will add children symbols to this when we encounter configurable declarations below.
    let mut configurable_symbol = token_map
        .tokens_for_file(uri)
        .find(|item| item.key().name == "configurable")
        .map(|item| {
            DocumentSymbolBuilder::new()
                .name(item.key().name.clone())
                .kind(lsp_types::SymbolKind::STRUCT)
                .range(item.key().range)
                .selection_range(item.key().range)
                .children(vec![])
                .build()
        });
    // Only include nodes that originate from the file.
    let mut nodes: Vec<_> = (if ty_program.root_module.span.source_id() == Some(&source_id) {
        Some(ty_program.root_module.all_nodes.iter())
    } else {
        ty_program
            .root_module
            .submodules_recursive()
            .find(|(_, submodule)| submodule.module.span.source_id() == Some(&source_id))
            .map(|(_, submodule)| submodule.module.all_nodes.iter())
    })
    .into_iter()
    .flatten()
    .filter_map(|node| {
        match &node.content {
            TyAstNodeContent::SideEffect(TySideEffect::IncludeStatement(include_statement)) => {
                Some(build_include_symbol(include_statement))
            }
            TyAstNodeContent::SideEffect(_) => None,
            TyAstNodeContent::Declaration(decl) => match decl {
                TyDecl::TypeAliasDecl(decl) => {
                    let type_alias_decl = engines.de().get_type_alias(&decl.decl_id);
                    let span = type_alias_decl.call_path.suffix.span();
                    let range = get_range_from_span(&span);
                    let detail = Some(type_alias_decl.ty.span().as_str().to_string());
                    let type_alias_symbol = DocumentSymbolBuilder::new()
                        .name(span.str().to_string())
                        .kind(lsp_types::SymbolKind::TYPE_PARAMETER)
                        .range(range)
                        .selection_range(range)
                        .detail(detail)
                        .build();
                    Some(type_alias_symbol)
                }
                TyDecl::FunctionDecl(decl) => {
                    let fn_decl = engines.de().get_function(&decl.decl_id);
                    let range = get_range_from_span(&fn_decl.name.span());
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
                    let decl_str = abi_decl.span().str();
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
                    let decl_str = trait_decl.span().str().to_string();
                    let name = extract_header(&decl_str);
                    let range = get_range_from_span(&trait_decl.name.span());
                    let children =
                        collect_interface_surface(engines, &trait_decl.interface_surface);
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
                    let decl_str = impl_trait_decl.span().str().to_string();
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
                    let configurable_decl = engines.de().get_configurable(&decl.decl_id);
                    let span = configurable_decl.call_path.suffix.span();
                    let range = get_range_from_span(&span);
                    let symbol = DocumentSymbolBuilder::new()
                        .name(span.str().to_string())
                        .kind(lsp_types::SymbolKind::FIELD)
                        .detail(Some(
                            configurable_decl
                                .type_ascription
                                .span()
                                .as_str()
                                .to_string(),
                        ))
                        .range(range)
                        .selection_range(range)
                        .build();
                    // Add symbol to the end of configurable_symbol's children field
                    configurable_symbol
                        .as_mut()?
                        .children
                        .as_mut()?
                        .push(symbol);
                    None
                }
                _ => None,
            },
            _ => None,
        }
    })
    .collect();

    // Add configurable symbol to the end after all children symbols have been added
    if let Some(symbol) = configurable_symbol {
        nodes.push(symbol);
    }

    // Sort by range start position
    nodes.sort_by_key(|node| node.range.start);
    nodes
}

fn build_include_symbol(include_statement: &TyIncludeStatement) -> DocumentSymbol {
    let span = include_statement.span();
    let range = get_range_from_span(&span);
    DocumentSymbolBuilder::new()
        .name(span.str().to_string())
        .kind(lsp_types::SymbolKind::MODULE)
        .range(range)
        .selection_range(range)
        .build()
}

fn build_constant_symbol(const_decl: &TyConstantDecl) -> DocumentSymbol {
    let span = const_decl.call_path.suffix.span();
    let range = get_range_from_span(&span);
    DocumentSymbolBuilder::new()
        .name(span.str().to_string())
        .kind(lsp_types::SymbolKind::CONSTANT)
        .detail(Some(const_decl.type_ascription.span().as_str().to_string()))
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

fn build_field_symbol(span: &Span, type_argument: &GenericArgument) -> DocumentSymbol {
    let range = get_range_from_span(span);
    DocumentSymbolBuilder::new()
        .name(span.clone().str().to_string())
        .detail(Some(type_argument.span().as_str().to_string()))
        .kind(lsp_types::SymbolKind::FIELD)
        .range(range)
        .selection_range(range)
        .build()
}

fn build_function_symbol(
    span: &Span,
    parameters: &[TyFunctionParameter],
    return_type: &GenericArgument,
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
            // Check for the presence of a CallPathTree, and if it exists, use the type information as the detail.
            let detail = variant
                .type_argument
                .call_path_tree()
                .as_ref()
                .map(|_| Some(variant.type_argument.span().as_str().to_string()))
                .unwrap_or(None);

            DocumentSymbolBuilder::new()
                .name(variant.name.span().str().to_string())
                .kind(lsp_types::SymbolKind::ENUM_MEMBER)
                .range(range)
                .selection_range(range)
                .detail(detail)
                .build()
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
                let type_name = format!("{}", engines.help_out(var_decl.type_ascription.type_id()));
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
fn fn_decl_detail(parameters: &[TyFunctionParameter], return_type: &GenericArgument) -> String {
    let params = parameters
        .iter()
        .map(|p| format!("{}: {}", p.name, p.type_argument.span().as_str()))
        .collect::<Vec<_>>()
        .join(", ");

    // Check for the presence of a CallPathTree, and if it exists, add it to the return type.
    let return_type = return_type
        .call_path_tree()
        .map(|_| format!(" -> {}", return_type.span().as_str()))
        .unwrap_or_default();
    format!("fn({params}){return_type}")
}

/// Extracts the header of a sway construct such as an `impl` block or `abi` declaration,
/// including any generic parameters, traits, or super traits, up to (but not including)
/// the opening `{` character. Trims any trailing whitespace.
///
/// If the `{` character is not found, the entire string is returned without trailing whitespace.
///
/// # Examples
///
/// ```ignore
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

impl Default for DocumentSymbolBuilder {
    fn default() -> Self {
        Self::new()
    }
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
