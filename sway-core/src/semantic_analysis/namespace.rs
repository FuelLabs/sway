use crate::{
    error::*, parse_tree::Visibility, type_engine::*, CallPath, CompileResult, Ident, TypeArgument,
    TypeInfo, TypeParameter, TypedDeclaration, TypedFunctionDeclaration,
};

use crate::semantic_analysis::{
    ast_node::{
        Monomorphizable, TypedExpression, TypedStorageDeclaration, TypedStructField,
        TypedVariableDeclaration,
    },
    declaration::{TypedStorageField, VariableMutability},
    TypeCheckedStorageAccess,
};

use sway_types::span::Span;

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

type ModuleName = String;
type TraitName = CallPath;
type SymbolMap = im::OrdMap<Ident, TypedDeclaration>;
type UseSynonyms = im::HashMap<Ident, Vec<Ident>>;
type UseAliases = im::HashMap<String, Ident>;

pub type Path = [Ident];
pub type PathBuf = Vec<Ident>;

/// The set of items that exist within some lexical scope via declaration or importing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Items {
    /// An ordered map from `Ident`s to their associated typed declarations.
    symbols: SymbolMap,
    implemented_traits: TraitMap,
    /// Represents the absolute path from which a symbol was imported.
    ///
    /// For example, in `use ::foo::bar::Baz;`, we store a mapping from the symbol `Baz` to its
    /// path `foo::bar::Baz`.
    use_synonyms: UseSynonyms,
    /// Represents an alternative name for an imported symbol.
    ///
    /// Aliases are introduced with syntax like `use foo::bar as baz;` syntax, where `baz` is an
    /// alias for `bar`.
    use_aliases: UseAliases,
    /// If there is a storage declaration (which are only valid in contracts), store it here.
    declared_storage: Option<TypedStorageDeclaration>,
}

/// A single `Module` within a Sway project.
///
/// A `Module` is most commonly associated with an individual file of Sway code, e.g. a top-level
/// script/predicate/contract file or some library dependency whether introduced via `dep` or the
/// `[dependencies]` table of a `forc` manifest.
///
/// A `Module` contains a set of all items that exist within the lexical scope via declaration or
/// importing, along with a map of each of its submodules.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Module {
    /// Submodules of the current module represented as an ordered map from each submodule's name
    /// to the associated `Module`.
    ///
    /// Submodules are normally introduced in Sway code with the `dep foo;` syntax where `foo` is
    /// some library dependency that we include as a submodule.
    ///
    /// Note that we *require* this map to be ordered to produce deterministic codegen results.
    submodules: im::OrdMap<ModuleName, Module>,
    /// The set of symbols, implementations, synonyms and aliases present within this module.
    items: Items,
}

/// The root module, from which all other modules can be accessed.
///
/// This is equivalent to the "crate root" of a Rust crate.
///
/// We use a custom type for the `Root` in order to ensure that methods that only work with
/// canonical paths, or that use canonical paths internally, are *only* called from the root. This
/// normally includes methods that first lookup some canonical path via `use_synonyms` before using
/// that canonical path to look up the symbol declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct Root {
    module: Module,
}

/// The set of items that represent the namespace context passed throughout type checking.
#[derive(Clone, Debug, PartialEq)]
pub struct Namespace {
    /// An immutable namespace that consists of the names that should always be present, no matter
    /// what module or scope we are currently checking.
    ///
    /// These include external library dependencies and (when it's added) the `std` prelude.
    ///
    /// This is passed through type-checking in order to initialise the namespace of each submodule
    /// within the project.
    init: Module,
    /// The `root` of the project namespace.
    ///
    /// From the root, the entirety of the project's namespace can always be accessed.
    ///
    /// The root is initialised from the `init` namespace before type-checking begins.
    root: Root,
    /// An absolute path from the `root` that represents the current module being checked.
    ///
    /// E.g. when type-checking the root module, this is equal to `[]`. When type-checking a
    /// submodule of the root called "foo", this would be equal to `[foo]`.
    mod_path: PathBuf,
}

/// A namespace session type representing the type-checking of a submodule.
///
/// This type allows for re-using the parent's `Namespace` in order to provide access to the
/// `root` and `init` throughout type-checking of the submodule, but with an updated `mod_path` to
/// represent the submodule's path. When dropped, the `SubmoduleNamespace` reset's the
/// `Namespace`'s `mod_path` to the parent module path so that type-checking of the parent may
/// continue.
pub struct SubmoduleNamespace<'a> {
    namespace: &'a mut Namespace,
    parent_mod_path: PathBuf,
}

impl Items {
    /// Immutable access to the inner symbol map.
    pub fn symbols(&self) -> &SymbolMap {
        &self.symbols
    }

    pub fn apply_storage_load(
        &self,
        fields: Vec<Ident>,
        storage_fields: &[TypedStorageField],
    ) -> CompileResult<(TypeCheckedStorageAccess, TypeId)> {
        match self.declared_storage {
            Some(ref storage) => storage.apply_storage_load(fields, storage_fields),
            None => err(
                vec![],
                vec![CompileError::NoDeclaredStorage {
                    span: fields[0].span().clone(),
                }],
            ),
        }
    }

    pub fn set_storage_declaration(&mut self, decl: TypedStorageDeclaration) -> CompileResult<()> {
        if self.declared_storage.is_some() {
            return err(
                vec![],
                vec![CompileError::MultipleStorageDeclarations { span: decl.span() }],
            );
        }
        self.declared_storage = Some(decl);
        ok((), vec![], vec![])
    }

    pub fn get_all_declared_symbols(&self) -> impl Iterator<Item = &TypedDeclaration> {
        self.symbols().values()
    }

    pub(crate) fn insert_symbol(
        &mut self,
        name: Ident,
        item: TypedDeclaration,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        // purposefully do not preemptively return errors so that the
        // new definiton allows later usages to compile
        if self.symbols.get(&name).is_some() {
            match item {
                TypedDeclaration::EnumDeclaration { .. }
                | TypedDeclaration::StructDeclaration { .. } => {
                    errors.push(CompileError::ShadowsOtherSymbol { name: name.clone() });
                }
                TypedDeclaration::GenericTypeForFunctionScope { .. } => {
                    errors.push(CompileError::GenericShadowsGeneric { name: name.clone() });
                }
                _ => {
                    warnings.push(CompileWarning {
                        span: name.span().clone(),
                        warning_content: Warning::ShadowsOtherSymbol { name: name.clone() },
                    });
                }
            }
        }
        self.symbols.insert(name, item);
        ok((), warnings, errors)
    }

    pub(crate) fn check_symbol(&self, name: &Ident) -> CompileResult<&TypedDeclaration> {
        match self.symbols.get(name) {
            Some(decl) => ok(decl, vec![], vec![]),
            None => err(
                vec![],
                vec![CompileError::SymbolNotFound { name: name.clone() }],
            ),
        }
    }

    pub(crate) fn insert_trait_implementation(
        &mut self,
        trait_name: CallPath,
        type_implementing_for: TypeInfo,
        functions_buf: Vec<TypedFunctionDeclaration>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let new_prefixes = if trait_name.prefixes.is_empty() {
            self.use_synonyms
                .get(&trait_name.suffix)
                .unwrap_or(&trait_name.prefixes)
                .clone()
        } else {
            trait_name.prefixes
        };
        let trait_name = CallPath {
            suffix: trait_name.suffix,
            prefixes: new_prefixes,
            is_absolute: trait_name.is_absolute,
        };
        check!(
            self.implemented_traits
                .insert(trait_name, type_implementing_for, functions_buf),
            (),
            warnings,
            errors
        );
        ok((), warnings, errors)
    }

    pub(crate) fn get_methods_for_type(&self, r#type: TypeId) -> Vec<TypedFunctionDeclaration> {
        self.implemented_traits
            .get_methods_for_type(look_up_type_id(r#type))
    }

    // Given a TypeInfo old_type with a set of methods available to it, make those same methods
    // available to TypeInfo new_type. This is useful in situations where old_type is being
    // monomorphized to new_type and and we want `get_methods_for_type()` to return the same set of
    // methods for new_type as it does for old_type.
    pub(crate) fn copy_methods_to_type(
        &mut self,
        old_type: TypeInfo,
        new_type: TypeInfo,
        type_mapping: &[(TypeParameter, TypeId)],
    ) {
        // This map grabs all (trait name, vec of methods) from self.implemented_traits
        // corresponding to `old_type`.
        let methods = self
            .implemented_traits
            .get_methods_for_type_by_trait(old_type);

        // Insert into `self.implemented_traits` the contents of the map above but with `new_type`
        // as the `TypeInfo` key.
        for (trait_name, mut trait_methods) in methods.into_iter() {
            trait_methods
                .iter_mut()
                .for_each(|method| method.copy_types(type_mapping));
            self.implemented_traits
                .insert(trait_name, new_type.clone(), trait_methods);
        }
    }

    pub(crate) fn get_tuple_elems(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<Vec<TypeArgument>> {
        let warnings = vec![];
        let errors = vec![];
        let ty = crate::type_engine::look_up_type_id(ty);
        match ty {
            TypeInfo::Tuple(elems) => ok(elems, warnings, errors),
            TypeInfo::ErrorRecovery => err(warnings, errors),
            a => err(
                vec![],
                vec![CompileError::NotATuple {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: a.friendly_type_str(),
                }],
            ),
        }
    }

    pub(crate) fn get_canonical_path(&self, symbol: &Ident) -> &[Ident] {
        self.use_synonyms.get(symbol).map(|v| &v[..]).unwrap_or(&[])
    }

    /// Given a declaration that may refer to a variable which contains a struct, find that
    /// struct's fields and name for use in determining if a subfield expression is valid.
    ///
    /// E.g. `foo.bar.baz`
    ///
    /// Is foo a struct? Does it contain a field bar? Is foo.bar a struct? Does `foo.bar` contain a
    /// field `baz`? This is the problem this function addresses.
    pub(crate) fn get_struct_type_fields(
        &self,
        ty: TypeId,
        debug_string: impl Into<String>,
        debug_span: &Span,
    ) -> CompileResult<(Vec<TypedStructField>, Ident)> {
        let ty = look_up_type_id(ty);
        match ty {
            TypeInfo::Struct { name, fields, .. } => ok((fields.to_vec(), name), vec![], vec![]),
            // If we hit `ErrorRecovery` then the source of that type should have populated
            // the error buffer elsewhere
            TypeInfo::ErrorRecovery => err(vec![], vec![]),
            a => err(
                vec![],
                vec![CompileError::NotAStruct {
                    name: debug_string.into(),
                    span: debug_span.clone(),
                    actually: a.friendly_type_str(),
                }],
            ),
        }
    }

    pub(crate) fn has_storage_declared(&self) -> bool {
        self.declared_storage.is_some()
    }

    pub(crate) fn get_storage_field_descriptors(&self) -> CompileResult<Vec<TypedStorageField>> {
        if let Some(fields) = self.declared_storage.as_ref().map(|ds| ds.fields.clone()) {
            ok(fields, vec![], vec![])
        } else {
            let msg = "unknown source location";
            let span = Span::new(Arc::from(msg), 0, msg.len(), None).unwrap();
            err(vec![], vec![CompileError::NoDeclaredStorage { span }])
        }
    }

    /// Returns a tuple where the first element is the [ResolvedType] of the actual expression, and
    /// the second is the [ResolvedType] of its parent, for control-flow analysis.
    pub(crate) fn find_subfield_type(
        &self,
        subfield_exp: &[Ident],
    ) -> CompileResult<(TypeId, TypeId)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut ident_iter = subfield_exp.iter().peekable();
        let first_ident = ident_iter.next().unwrap();
        let symbol = match self.symbols.get(first_ident).cloned() {
            Some(s) => s,
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: first_ident.clone(),
                });
                return err(warnings, errors);
            }
        };
        if ident_iter.peek().is_none() {
            let ty = check!(
                symbol.return_type(),
                return err(warnings, errors),
                warnings,
                errors
            );
            return ok((ty, ty), warnings, errors);
        }
        let mut symbol = check!(
            symbol.return_type(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut type_fields =
            self.get_struct_type_fields(symbol, first_ident.as_str(), first_ident.span());
        warnings.append(&mut type_fields.warnings);
        errors.append(&mut type_fields.errors);
        let (mut fields, struct_name): (Vec<TypedStructField>, Ident) = match type_fields.value {
            // if it is missing, the error message comes from within the above method
            // so we don't need to re-add it here
            None => return err(warnings, errors),
            Some(value) => value,
        };

        let mut parent_rover = symbol;

        for ident in ident_iter {
            // find the ident in the currently available fields
            let TypedStructField { r#type, .. } =
                match fields.iter().find(|x| x.name.as_str() == ident.as_str()) {
                    Some(field) => field.clone(),
                    None => {
                        // gather available fields for the error message
                        let available_fields =
                            fields.iter().map(|x| x.name.as_str()).collect::<Vec<_>>();

                        errors.push(CompileError::FieldNotFound {
                            field_name: ident.clone(),
                            struct_name,
                            available_fields: available_fields.join(", "),
                        });
                        return err(warnings, errors);
                    }
                };

            match look_up_type_id(r#type) {
                TypeInfo::Struct {
                    fields: ref l_fields,
                    ..
                } => {
                    parent_rover = symbol;
                    fields = l_fields.clone();
                    symbol = r#type;
                }
                _ => {
                    fields = vec![];
                    parent_rover = symbol;
                    symbol = r#type;
                }
            }
        }
        ok((symbol, parent_rover), warnings, errors)
    }
}

impl Module {
    /// Immutable access to this module's submodules.
    pub fn submodules(&self) -> &im::OrdMap<ModuleName, Module> {
        &self.submodules
    }

    /// Insert a submodule into this `Module`.
    pub fn insert_submodule(&mut self, name: String, submodule: Module) {
        self.submodules.insert(name, submodule);
    }

    /// Lookup the submodule at the given path.
    pub fn submodule(&self, path: &Path) -> Option<&Module> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Unique access to the submodule at the given path.
    pub fn submodule_mut(&mut self, path: &Path) -> Option<&mut Module> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get_mut(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Lookup the submodule at the given path.
    ///
    /// This should be used rather than `Index` when we don't yet know whether the module exists.
    pub(crate) fn check_submodule(&self, path: &[Ident]) -> CompileResult<&Module> {
        match self.submodule(path) {
            None => err(vec![], vec![module_not_found(path)]),
            Some(module) => ok(module, vec![], vec![]),
        }
    }

    /// Find the submodule at the given path relative to this module and return mutable access to
    /// it.
    ///
    /// This should be used rather than `IndexMut` when we don't yet know whether the module
    /// exists.
    pub(crate) fn check_submodule_mut(&mut self, path: &[Ident]) -> CompileResult<&mut Module> {
        match self.submodule_mut(path) {
            None => err(vec![], vec![module_not_found(path)]),
            Some(module) => ok(module, vec![], vec![]),
        }
    }

    /// Given a path to a `src` module, create synonyms to every symbol in that module to the given
    /// `dst` module.
    ///
    /// This is used when an import path contains an asterisk.
    ///
    /// Paths are assumed to be relative to `self`.
    pub(crate) fn star_import(&mut self, src: &Path, dst: &Path) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let src_ns = check!(
            self.check_submodule(src),
            return err(warnings, errors),
            warnings,
            errors
        );
        let implemented_traits = src_ns.implemented_traits.clone();
        let symbols = src_ns
            .symbols
            .iter()
            .filter_map(|(symbol, decl)| {
                if decl.visibility() == Visibility::Public {
                    Some(symbol.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let dst_ns = &mut self[dst];
        check!(
            dst_ns.implemented_traits.extend(implemented_traits),
            (),
            warnings,
            errors
        );
        for symbol in symbols {
            if dst_ns.use_synonyms.contains_key(&symbol) {
                errors.push(CompileError::StarImportShadowsOtherSymbol {
                    name: symbol.clone(),
                });
            }
            dst_ns.use_synonyms.insert(symbol, src.to_vec());
        }
        ok((), warnings, errors)
    }

    /// Pull a single item from a `src` module and import it into the `dst` module.
    ///
    /// The item we want to import is basically the last item in path because this is a `self`
    /// import.
    pub(crate) fn self_import(
        &mut self,
        src: &Path,
        dst: &Path,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        let (last_item, src) = src.split_last().expect("guaranteed by grammar");
        self.item_import(src, last_item, dst, alias)
    }

    /// Pull a single `item` from the given `src` module and import it into the `dst` module.
    ///
    /// Paths are assumed to be relative to `self`.
    pub(crate) fn item_import(
        &mut self,
        src: &Path,
        item: &Ident,
        dst: &Path,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let src_ns = check!(
            self.check_submodule(src),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut impls_to_insert = vec![];
        match src_ns.symbols.get(item).cloned() {
            Some(decl) => {
                if decl.visibility() != Visibility::Public {
                    errors.push(CompileError::ImportPrivateSymbol { name: item.clone() });
                }
                // if this is a const, insert it into the local namespace directly
                if let TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    is_mutable: VariableMutability::ExportedConst,
                    ref name,
                    ..
                }) = decl
                {
                    self[dst].insert_symbol(alias.unwrap_or_else(|| name.clone()), decl.clone());
                    return ok((), warnings, errors);
                }
                let a = decl.return_type().value;
                //  if this is an enum or struct, import its implementations
                let mut res = match a {
                    Some(a) => src_ns
                        .implemented_traits
                        .get_call_path_and_type_info(look_up_type_id(a)),
                    None => vec![],
                };
                impls_to_insert.append(&mut res);
                // no matter what, import it this way though.
                let dst_ns = &mut self[dst];
                match alias {
                    Some(alias) => {
                        if dst_ns.use_synonyms.contains_key(&alias) {
                            errors.push(CompileError::ShadowsOtherSymbol {
                                name: alias.clone(),
                            });
                        }
                        dst_ns.use_synonyms.insert(alias.clone(), src.to_vec());
                        dst_ns
                            .use_aliases
                            .insert(alias.as_str().to_string(), item.clone());
                    }
                    None => {
                        if dst_ns.use_synonyms.contains_key(item) {
                            errors.push(CompileError::ShadowsOtherSymbol { name: item.clone() });
                        }
                        dst_ns.use_synonyms.insert(item.clone(), src.to_vec());
                    }
                };
            }
            None => {
                errors.push(CompileError::SymbolNotFound { name: item.clone() });
                return err(warnings, errors);
            }
        };

        let dst_ns = &mut self[dst];
        impls_to_insert
            .into_iter()
            .for_each(|((call_path, type_info), methods)| {
                dst_ns
                    .implemented_traits
                    .insert(call_path, type_info, methods);
            });

        ok((), warnings, errors)
    }
}

impl Root {
    /// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
    ///
    /// This is short-hand for concatenating the `mod_path` with the `call_path`'s prefixes and
    /// then calling `resolve_symbol` with the resulting path and call_path's suffix.
    pub(crate) fn resolve_call_path(
        &self,
        mod_path: &Path,
        call_path: &CallPath,
    ) -> CompileResult<&TypedDeclaration> {
        let symbol_path: Vec<_> = mod_path
            .iter()
            .chain(&call_path.prefixes)
            .cloned()
            .collect();
        self.resolve_symbol(&symbol_path, &call_path.suffix)
    }

    /// Given a path to a module and the identifier of a symbol within that module, resolve its
    /// declaration.
    ///
    /// If the symbol is within the given module's namespace via import, we recursively traverse
    /// imports until we find the original declaration.
    pub(crate) fn resolve_symbol(
        &self,
        mod_path: &Path,
        symbol: &Ident,
    ) -> CompileResult<&TypedDeclaration> {
        self.check_submodule(mod_path).flat_map(|module| {
            let true_symbol = self[mod_path]
                .use_aliases
                .get(symbol.as_str())
                .unwrap_or(symbol);
            match module.use_synonyms.get(symbol) {
                Some(src_path) if mod_path != src_path => {
                    self.resolve_symbol(src_path, true_symbol)
                }
                _ => module.check_symbol(true_symbol),
            }
        })
    }

    /// This function either returns a struct (i.e. custom type), `None`, denoting the type that is
    /// being looked for is actually a generic, not-yet-resolved type.
    ///
    /// If a self type is given and anything on this ref chain refers to self, update the chain.
    pub(crate) fn resolve_type_with_self(
        &mut self,
        mod_path: &Path,
        ty: TypeInfo,
        self_type: TypeId,
        span: Span,
        enforce_type_args: bool,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_id = match ty {
            TypeInfo::Custom {
                ref name,
                type_arguments,
            } => {
                let mut new_type_arguments = vec![];
                for type_argument in type_arguments.into_iter() {
                    let new_type_id = check!(
                        self.resolve_type_with_self(
                            mod_path,
                            look_up_type_id(type_argument.type_id),
                            self_type,
                            type_argument.span.clone(),
                            enforce_type_args
                        ),
                        insert_type(TypeInfo::ErrorRecovery),
                        warnings,
                        errors
                    );
                    let type_argument = TypeArgument {
                        type_id: new_type_id,
                        span: type_argument.span,
                    };
                    new_type_arguments.push(type_argument);
                }
                match self
                    .resolve_symbol(mod_path, name)
                    .ok(&mut warnings, &mut errors)
                    .cloned()
                {
                    Some(TypedDeclaration::StructDeclaration(decl)) => {
                        if enforce_type_args
                            && new_type_arguments.is_empty()
                            && !decl.type_parameters.is_empty()
                        {
                            errors.push(CompileError::NeedsTypeArguments {
                                name: name.clone(),
                                span: name.span().clone(),
                            });
                            return err(warnings, errors);
                        }
                        if !decl.type_parameters.is_empty() {
                            let new_decl = check!(
                                decl.monomorphize(
                                    &mut self[mod_path],
                                    &new_type_arguments,
                                    Some(self_type)
                                ),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            new_decl.type_id()
                        } else {
                            decl.type_id()
                        }
                    }
                    Some(TypedDeclaration::EnumDeclaration(decl)) => {
                        if enforce_type_args
                            && new_type_arguments.is_empty()
                            && !decl.type_parameters.is_empty()
                        {
                            errors.push(CompileError::NeedsTypeArguments {
                                name: name.clone(),
                                span: name.span().clone(),
                            });
                            return err(warnings, errors);
                        }
                        if !decl.type_parameters.is_empty() {
                            let new_decl = check!(
                                decl.monomorphize(
                                    &mut self[mod_path],
                                    &new_type_arguments,
                                    Some(self_type)
                                ),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            new_decl.type_id()
                        } else {
                            decl.type_id()
                        }
                    }
                    Some(TypedDeclaration::GenericTypeForFunctionScope { name, .. }) => {
                        insert_type(TypeInfo::UnknownGeneric { name })
                    }
                    _ => {
                        errors.push(CompileError::UnknownType { span });
                        return err(warnings, errors);
                    }
                }
            }
            TypeInfo::Array(type_id, size) => {
                let elem_type_id = check!(
                    self.resolve_type_with_self(
                        mod_path,
                        look_up_type_id(type_id),
                        self_type,
                        span,
                        enforce_type_args
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );
                insert_type(TypeInfo::Array(elem_type_id, size))
            }
            TypeInfo::SelfType => self_type,
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        };
        ok(type_id, warnings, errors)
    }

    pub(crate) fn resolve_type_without_self(
        &mut self,
        mod_path: &Path,
        ty: &TypeInfo,
    ) -> CompileResult<TypeId> {
        let ty = ty.clone();
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_id = match ty {
            TypeInfo::Custom {
                name,
                type_arguments,
            } => match self
                .resolve_symbol(mod_path, &name)
                .ok(&mut warnings, &mut errors)
                .cloned()
            {
                Some(TypedDeclaration::StructDeclaration(decl)) => {
                    let mut new_type_arguments = vec![];
                    for type_argument in type_arguments.into_iter() {
                        let new_type_id = check!(
                            self.resolve_type_without_self(
                                mod_path,
                                &look_up_type_id(type_argument.type_id),
                            ),
                            insert_type(TypeInfo::ErrorRecovery),
                            warnings,
                            errors
                        );
                        let type_argument = TypeArgument {
                            type_id: new_type_id,
                            span: type_argument.span,
                        };
                        new_type_arguments.push(type_argument);
                    }
                    if !decl.type_parameters.is_empty() {
                        let new_decl = check!(
                            decl.monomorphize(&mut self[mod_path], &new_type_arguments, None),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        new_decl.type_id()
                    } else {
                        decl.type_id()
                    }
                }
                Some(TypedDeclaration::EnumDeclaration(decl)) => {
                    let mut new_type_arguments = vec![];
                    for type_argument in type_arguments.into_iter() {
                        let new_type_id = check!(
                            self.resolve_type_without_self(
                                mod_path,
                                &look_up_type_id(type_argument.type_id),
                            ),
                            insert_type(TypeInfo::ErrorRecovery),
                            warnings,
                            errors
                        );
                        let type_argument = TypeArgument {
                            type_id: new_type_id,
                            span: type_argument.span,
                        };
                        new_type_arguments.push(type_argument);
                    }
                    if !decl.type_parameters.is_empty() {
                        let new_decl = check!(
                            decl.monomorphize(&mut self[mod_path], &new_type_arguments, None),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        new_decl.type_id()
                    } else {
                        decl.type_id()
                    }
                }
                _ => insert_type(TypeInfo::Unknown),
            },
            TypeInfo::Array(type_id, size) => {
                let elem_type_id = check!(
                    self.resolve_type_without_self(mod_path, &look_up_type_id(type_id)),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );
                insert_type(TypeInfo::Array(elem_type_id, size))
            }
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        };
        ok(type_id, warnings, errors)
    }

    /// Given a method and a type (plus a `self_type` to potentially resolve it), find that method
    /// in the namespace. Requires `args_buf` because of some special casing for the standard
    /// library where we pull the type from the arguments buffer.
    ///
    /// This function will generate a missing method error if the method is not found.
    ///
    /// This method should only be called on the root namespace. `mod_path` is the current module,
    /// `method_path` is assumed to be absolute.
    pub(crate) fn find_method_for_type(
        &mut self,
        mod_path: &Path,
        r#type: TypeId,
        method_path: &Path,
        self_type: TypeId,
        args_buf: &VecDeque<TypedExpression>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let local_methods = self[mod_path].get_methods_for_type(r#type);
        let (method_name, method_prefix) = method_path.split_last().expect("method path is empty");

        // Ensure there's a module for the given method prefix.
        check!(
            self.check_submodule(method_prefix),
            return err(warnings, errors),
            warnings,
            errors
        );

        // This is a hack and I don't think it should be used.  We check the local namespace first,
        // but if nothing turns up then we try the namespace where the type itself is declared.
        let r#type = check!(
            self.resolve_type_with_self(
                method_prefix,
                look_up_type_id(r#type),
                self_type,
                method_name.span().clone(),
                false
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );
        let mut ns_methods = self[method_prefix].get_methods_for_type(r#type);

        let mut methods = local_methods;
        methods.append(&mut ns_methods);

        match methods
            .into_iter()
            .find(|TypedFunctionDeclaration { name, .. }| name == method_name)
        {
            Some(o) => ok(o, warnings, errors),
            None => {
                if args_buf.get(0).map(|x| look_up_type_id(x.return_type))
                    != Some(TypeInfo::ErrorRecovery)
                {
                    errors.push(CompileError::MethodNotFound {
                        method_name: method_name.clone(),
                        type_name: r#type.friendly_type_str(),
                    });
                }
                err(warnings, errors)
            }
        }
    }
}

impl Namespace {
    /// Initialise the namespace at its root from the given initial namespace.
    pub fn init_root(init: Module) -> Self {
        let root = Root::from(init.clone());
        let mod_path = vec![];
        Self {
            init,
            root,
            mod_path,
        }
    }

    /// A reference to the path of the module currently being type-checked.
    pub fn mod_path(&self) -> &Path {
        &self.mod_path
    }

    /// A reference to the root of the project namespace.
    pub fn root(&self) -> &Root {
        &self.root
    }

    /// Access to the current [Module], i.e. the module at the inner `mod_path`.
    ///
    /// Note that the [Namespace] will automatically dereference to this [Module] when attempting
    /// to call any [Module] methods.
    pub fn module(&self) -> &Module {
        &self.root.module[&self.mod_path]
    }

    /// Mutable access to the current [Module], i.e. the module at the inner `mod_path`.
    ///
    /// Note that the [Namespace] will automatically dereference to this [Module] when attempting
    /// to call any [Module] methods.
    pub fn module_mut(&mut self) -> &mut Module {
        &mut self.root.module[&self.mod_path]
    }

    /// Short-hand for calling [Root::resolve_symbol] on `root` with the `mod_path`.
    pub(crate) fn resolve_symbol(&self, symbol: &Ident) -> CompileResult<&TypedDeclaration> {
        self.root.resolve_symbol(&self.mod_path, symbol)
    }

    /// Short-hand for calling [Root::resolve_call_path] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path(
        &self,
        call_path: &CallPath,
    ) -> CompileResult<&TypedDeclaration> {
        self.root.resolve_call_path(&self.mod_path, call_path)
    }

    /// Short-hand for calling [Root::resolve_type_with_self] on `root` with the `mod_path`.
    pub(crate) fn resolve_type_with_self(
        &mut self,
        ty: TypeInfo,
        self_type: TypeId,
        span: Span,
        enforce_type_args: bool,
    ) -> CompileResult<TypeId> {
        self.root
            .resolve_type_with_self(&self.mod_path, ty, self_type, span, enforce_type_args)
    }

    /// Short-hand for calling [Root::find_method_for_type] on `root` with the `mod_path`.
    pub(crate) fn find_method_for_type(
        &mut self,
        r#type: TypeId,
        method_path: &Path,
        self_type: TypeId,
        args_buf: &VecDeque<TypedExpression>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        self.root
            .find_method_for_type(&self.mod_path, r#type, method_path, self_type, args_buf)
    }

    /// Short-hand for calling [Root::resolve_type_without_self] on `root` and with the `mod_path`.
    pub(crate) fn resolve_type_without_self(&mut self, ty: &TypeInfo) -> CompileResult<TypeId> {
        self.root.resolve_type_without_self(&self.mod_path, ty)
    }

    /// Short-hand for performing a [Module::star_import] with `mod_path` as the destination.
    pub(crate) fn star_import(&mut self, src: &Path) -> CompileResult<()> {
        self.root.star_import(src, &self.mod_path)
    }

    /// Short-hand for performing a [Module::self_import] with `mod_path` as the destination.
    pub(crate) fn self_import(&mut self, src: &Path, alias: Option<Ident>) -> CompileResult<()> {
        self.root.self_import(src, &self.mod_path, alias)
    }

    /// Short-hand for performing a [Module::item_import] with `mod_path` as the destination.
    pub(crate) fn item_import(
        &mut self,
        src: &Path,
        item: &Ident,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        self.root.item_import(src, item, &self.mod_path, alias)
    }

    /// "Enter" the submodule at the given path by returning a new [SubmoduleNamespace].
    ///
    /// Here we temporarily change `mod_path` to the given `dep_mod_path` and wrap `self` in a
    /// [SubmoduleNamespace] type. When dropped, the [SubmoduleNamespace] resets the `mod_path`
    /// back to the original path so that we can continue type-checking the current module after
    /// finishing with the dependency.
    pub(crate) fn enter_submodule(&mut self, dep_name: Ident) -> SubmoduleNamespace {
        let init = self.init.clone();
        self.submodules.entry(dep_name.to_string()).or_insert(init);
        let submod_path: Vec<_> = self
            .mod_path
            .iter()
            .cloned()
            .chain(Some(dep_name))
            .collect();
        let parent_mod_path = std::mem::replace(&mut self.mod_path, submod_path);
        SubmoduleNamespace {
            namespace: self,
            parent_mod_path,
        }
    }
}

impl std::ops::Deref for Module {
    type Target = Items;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl std::ops::DerefMut for Module {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl std::ops::Deref for Root {
    type Target = Module;
    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl std::ops::DerefMut for Root {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.module
    }
}

impl std::ops::Deref for Namespace {
    type Target = Module;
    fn deref(&self) -> &Self::Target {
        self.module()
    }
}

impl std::ops::DerefMut for Namespace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.module_mut()
    }
}

impl<'a> std::ops::Deref for SubmoduleNamespace<'a> {
    type Target = Namespace;
    fn deref(&self) -> &Self::Target {
        self.namespace
    }
}

impl<'a> std::ops::DerefMut for SubmoduleNamespace<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.namespace
    }
}

impl<'a> std::ops::Index<&'a Path> for Module {
    type Output = Module;
    fn index(&self, path: &'a Path) -> &Self::Output {
        self.submodule(path)
            .unwrap_or_else(|| panic!("no module for the given path {:?}", path))
    }
}

impl<'a> std::ops::IndexMut<&'a Path> for Module {
    fn index_mut(&mut self, path: &'a Path) -> &mut Self::Output {
        self.submodule_mut(path)
            .unwrap_or_else(|| panic!("no module for the given path {:?}", path))
    }
}

impl From<Module> for Root {
    fn from(module: Module) -> Self {
        Root { module }
    }
}

impl From<Root> for Module {
    fn from(root: Root) -> Self {
        root.module
    }
}

impl From<Namespace> for Root {
    fn from(namespace: Namespace) -> Self {
        namespace.root
    }
}

impl<'a> Drop for SubmoduleNamespace<'a> {
    fn drop(&mut self) {
        // Replace the submodule path with the original module path.
        // This ensures that the namespace's module path is reset when ownership over it is
        // relinquished from the SubmoduleNamespace.
        self.namespace.mod_path = std::mem::take(&mut self.parent_mod_path);
    }
}

// This cannot be a HashMap because of how TypeInfo's are handled.
//
// In Rust, in general, a custom type should uphold the invariant
// that PartialEq and Hash produce consistent results. i.e. for
// two objects, their hash value is equal if and only if they are
// equal under the PartialEq trait.
//
// For TypeInfo, this means that if you have:
//
// ```ignore
// 1: u64
// 2: u64
// 3: Ref(1)
// 4: Ref(2)
// ```
//
// 1, 2, 3, 4 are equal under PartialEq and their hashes are the same
// value.
//
// However, we need this structure to be able to maintain the
// difference between 3 and 4, as in practice, 1 and 2 might not yet
// be resolved.
type TraitMapInner = im::Vector<((TraitName, TypeInfo), TraitMethods)>;
type TraitMethods = im::HashMap<String, TypedFunctionDeclaration>;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TraitMap {
    trait_map: TraitMapInner,
}

impl TraitMap {
    pub(crate) fn insert(
        &mut self,
        trait_name: CallPath,
        type_implementing_for: TypeInfo,
        methods: Vec<TypedFunctionDeclaration>,
    ) -> CompileResult<()> {
        let warnings = vec![];
        let errors = vec![];
        let mut methods_map = im::HashMap::new();
        for method in methods.into_iter() {
            let method_name = method.name.as_str().to_string();
            methods_map.insert(method_name, method);
        }
        self.trait_map
            .push_back(((trait_name, type_implementing_for), methods_map));
        ok((), warnings, errors)
    }

    pub(crate) fn extend(&mut self, other: TraitMap) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        for ((trait_name, type_implementing_for), methods) in other.trait_map.into_iter() {
            check!(
                self.insert(
                    trait_name,
                    type_implementing_for,
                    methods.values().cloned().collect()
                ),
                (),
                warnings,
                errors
            );
        }
        ok((), warnings, errors)
    }

    pub(crate) fn get_call_path_and_type_info(
        &self,
        r#type: TypeInfo,
    ) -> Vec<((CallPath, TypeInfo), Vec<TypedFunctionDeclaration>)> {
        let mut ret = vec![];
        for ((call_path, type_info), methods) in self.trait_map.iter() {
            if type_info.clone() == r#type {
                ret.push((
                    (call_path.clone(), type_info.clone()),
                    methods.values().cloned().collect(),
                ));
            }
        }
        ret
    }

    fn get_methods_for_type(&self, r#type: TypeInfo) -> Vec<TypedFunctionDeclaration> {
        let mut methods = vec![];
        for ((_, type_info), l_methods) in self.trait_map.iter() {
            if *type_info == r#type {
                methods.append(&mut l_methods.values().cloned().collect());
            }
        }
        methods
    }

    fn get_methods_for_type_by_trait(
        &self,
        r#type: TypeInfo,
    ) -> HashMap<TraitName, Vec<TypedFunctionDeclaration>> {
        let mut methods: HashMap<TraitName, Vec<TypedFunctionDeclaration>> = HashMap::new();
        for ((trait_name, type_info), trait_methods) in self.trait_map.iter() {
            if *type_info == r#type {
                methods.insert(
                    (*trait_name).clone(),
                    trait_methods.values().cloned().collect(),
                );
            }
        }
        methods
    }
}

fn module_not_found(path: &[Ident]) -> CompileError {
    CompileError::ModuleNotFound {
        span: path.iter().fold(path[0].span().clone(), |acc, this_one| {
            if acc.path() == this_one.span().path() {
                Span::join(acc, this_one.span().clone())
            } else {
                acc
            }
        }),
        name: path
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join("::"),
    }
}
