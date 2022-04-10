use crate::{
    error::*, parse_tree::Visibility, type_engine::*, CallPath, CompileResult, Ident, TypeArgument,
    TypeInfo, TypeParameter, TypedDeclaration, TypedFunctionDeclaration,
};

use crate::semantic_analysis::{
    ast_node::{TypedEnumDeclaration, TypedExpression, TypedStorageDeclaration, TypedStructField, TypedVariableDeclaration},
    declaration::{TypedStorageField, VariableMutability},
    TypeCheckedStorageAccess,
};

use sway_types::span::Span;

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    sync::Arc,
};

type ModuleName = String;
type TraitName = CallPath;

/// A namespace represents all items that exist either within some lexical scope via declaration or
/// importing.
///
/// The namespace is constructed during type checking.
#[derive(Clone, Debug, Default)]
pub struct Namespace {
    // This is a BTreeMap because we rely on its ordering being consistent. See
    // [Namespace::get_all_declared_symbols] -- we need that iterator to have a deterministic
    // order.
    symbols: BTreeMap<Ident, TypedDeclaration>,
    implemented_traits: TraitMap,
    // Any other modules within this scope, where a module is a namespace associated with an identifier.
    // This is a BTreeMap because we rely on its ordering being consistent. See
    // [Namespace::get_all_imported_modules] -- we need that iterator to have a deterministic
    // order.
    modules: BTreeMap<ModuleName, Namespace>,
    use_synonyms: HashMap<Ident, Vec<Ident>>,
    /// Represents an alternative name for a symbol.
    use_aliases: HashMap<String, Ident>,
    /// If there is a storage declaration (which are only valid in contracts), store it here.
    declared_storage: Option<TypedStorageDeclaration>,
}

impl Namespace {
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
        self.symbols.values()
    }

    pub fn get_all_imported_modules(&self) -> impl Iterator<Item = &Namespace> {
        self.modules.values()
    }

    pub(crate) fn insert(&mut self, name: Ident, item: TypedDeclaration) -> CompileResult<()> {
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

    pub fn insert_module(&mut self, module_name: String, ix: Namespace) {
        self.modules.insert(module_name, ix);
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
        type_mapping: &[(TypeParameter, usize)],
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

    /// Used for calls that look like this:
    ///
    /// `foo::bar::function`
    ///
    /// where `foo` and `bar` are the prefixes and `function` is the suffix.
    pub(crate) fn get_call_path(&self, symbol: &CallPath) -> CompileResult<TypedDeclaration> {
        let path = if symbol.prefixes.is_empty() {
            self.use_synonyms
                .get(&symbol.suffix)
                .unwrap_or(&symbol.prefixes)
        } else {
            &symbol.prefixes
        };
        self.get_name_from_path(path, &symbol.suffix)
    }

    pub(crate) fn get_canonical_path(&self, symbol: &Ident) -> &[Ident] {
        self.use_synonyms.get(symbol).map(|v| &v[..]).unwrap_or(&[])
    }

    pub(crate) fn get_name_from_path(
        &self,
        path: &[Ident],
        name: &Ident,
    ) -> CompileResult<TypedDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let module = check!(
            self.find_module_relative(path),
            return err(warnings, errors),
            warnings,
            errors
        );
        match module.symbols.get(name).cloned() {
            Some(decl) => ok(decl, warnings, errors),
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: name.as_str().to_string(),
                    span: name.span().clone(),
                });
                err(warnings, errors)
            }
        }
    }

    pub(crate) fn find_module_relative(&self, path: &[Ident]) -> CompileResult<&Namespace> {
        let mut errors = vec![];
        let warnings = vec![];
        if path.is_empty() {
            return ok(self, warnings, errors);
        }
        let mut mod_ns = match self.modules.get(path[0].as_str()) {
            Some(ns) => ns,
            None => {
                errors.push(CompileError::ModuleNotFound {
                    span: path.iter().fold(path[0].span().clone(), |acc, this_one| {
                        Span::join(acc, this_one.span().clone())
                    }),
                    name: path
                        .iter()
                        .map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join("::"),
                });
                return err(warnings, errors);
            }
        };
        for ident in path.iter().skip(1) {
            match mod_ns.modules.get(ident.as_str()) {
                Some(ns) => mod_ns = ns,
                None => {
                    errors.push(CompileError::ModuleNotFound {
                        span: path.iter().fold(path[0].span().clone(), |acc, this_one| {
                            Span::join(acc, this_one.span().clone())
                        }),
                        name: path
                            .iter()
                            .map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .join("::"),
                    });
                    return err(warnings, errors);
                }
            }
        }
        ok(mod_ns, warnings, errors)
    }

    /// This function either returns a struct (i.e. custom type), `None`, denoting the type that is
    /// being looked for is actually a generic, not-yet-resolved type.
    ///
    /// If a self type is given and anything on this ref chain refers to self, update the chain.
    pub(crate) fn resolve_type_with_self(
        &mut self,
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
                        Self::resolve_type_with_self(
                            self,
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
                match self.get_symbol(name).ok(&mut warnings, &mut errors) {
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
                                decl.monomorphize(self, &new_type_arguments, Some(self_type)),
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
                                decl.monomorphize_with_type_arguments(
                                    self,
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
            TypeInfo::SelfType => self_type,
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        };
        ok(type_id, warnings, errors)
    }

    pub(crate) fn resolve_type_without_self(&mut self, ty: &TypeInfo) -> CompileResult<TypeId> {
        let ty = ty.clone();
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_id = match ty {
            TypeInfo::Custom {
                name,
                type_arguments,
            } => match self.get_symbol(&name).ok(&mut warnings, &mut errors) {
                Some(TypedDeclaration::StructDeclaration(decl)) => {
                    let mut new_type_arguments = vec![];
                    for type_argument in type_arguments.into_iter() {
                        let new_type_id = check!(
                            Self::resolve_type_without_self(
                                self,
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
                            decl.monomorphize(self, &new_type_arguments, None),
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
                            Self::resolve_type_without_self(
                                self,
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
                            decl.monomorphize_with_type_arguments(self, &new_type_arguments, None),
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
            TypeInfo::Ref(id) => id,
            o => insert_type(o),
        };
        ok(type_id, warnings, errors)
    }

    pub(crate) fn get_symbol(&self, symbol: &Ident) -> CompileResult<TypedDeclaration> {
        let empty = vec![];
        let path = self.use_synonyms.get(symbol).unwrap_or(&empty);
        let true_symbol = self
            .use_aliases
            .get(&symbol.as_str().to_string())
            .unwrap_or(symbol);
        self.get_name_from_path(&path, &true_symbol)
    }

    /// Given a method and a type (plus a `self_type` to potentially resolve it), find that method
    /// in the namespace. Requires `args_buf` because of some special casing for the standard
    /// library where we pull the type from the arguments buffer.
    ///
    /// This function will generate a missing method error if the method is not found.
    pub(crate) fn find_method_for_type(
        &self,
        r#type: TypeId,
        method_name: &Ident,
        method_path: &[Ident],
        from_module: Option<&Namespace>,
        self_type: TypeId,
        args_buf: &VecDeque<TypedExpression>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let base_module = from_module.unwrap_or(self);
        let mut namespace = check!(
            base_module.find_module_relative(method_path).map(|ns| ns.clone()),
            return err(warnings, errors),
            warnings,
            errors
        );

        // This is a hack and I don't think it should be used.  We check the local namespace first,
        // but if nothing turns up then we try the namespace where the type itself is declared.
        let r#type = check!(
            namespace.resolve_type_with_self(
                look_up_type_id(r#type),
                self_type,
                method_name.span().clone(),
                false
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );
        let local_methods = self.get_methods_for_type(r#type);
        let mut ns_methods = namespace.get_methods_for_type(r#type);
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
                        method_name: method_name.as_str().to_string(),
                        type_name: r#type.friendly_type_str(),
                        span: method_name.span().clone(),
                    });
                }
                err(warnings, errors)
            }
        }
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
                    var_name: first_ident.as_str().to_string(),
                    span: first_ident.span().clone(),
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
                            struct_name: struct_name.to_string(),
                            available_fields: available_fields.join(", "),
                            span: ident.span().clone(),
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

    pub(crate) fn find_enum(&self, enum_name: &Ident) -> Option<TypedEnumDeclaration> {
        match self.get_symbol(enum_name) {
            CompileResult {
                value: Some(TypedDeclaration::EnumDeclaration(inner)),
                ..
            } => Some(inner),
            _ => None,
        }
    }

    /// Given a path to a module, create synonyms to every symbol in that module.
    /// This is used when an import path contains an asterisk.
    pub(crate) fn star_import(
        &mut self,
        from_module: Option<&Namespace>,
        path: Vec<Ident>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let namespace = {
            let base_namespace = from_module.unwrap_or(self);
            check!(
                base_namespace.find_module_relative(&path),
                return err(warnings, errors),
                warnings,
                errors
            )
        };
        let implemented_traits = namespace.implemented_traits.clone();
        let symbols = namespace
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

        check!(
            self.implemented_traits.extend(implemented_traits),
            (),
            warnings,
            errors
        );
        for symbol in symbols {
            if self.use_synonyms.contains_key(&symbol) {
                errors.push(CompileError::StarImportShadowsOtherSymbol {
                    name: symbol.as_str().to_string(),
                    span: symbol.span().clone(),
                });
            }
            self.use_synonyms.insert(symbol, path.clone());
        }
        ok((), warnings, errors)
    }

    /// Pull a single item from a module and import it into this namespace.
    ///
    /// The item we want to import is basically the last item in path because this is a self
    /// import.
    pub(crate) fn self_import(
        &mut self,
        from_namespace: Option<&Namespace>,
        path: Vec<Ident>,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        let mut new_path = path;
        let last_item = new_path.pop().expect("guaranteed by grammar");
        self.item_import(from_namespace, new_path, &last_item, alias)
    }

    /// Pull a single item from a module and import it into this namespace.
    pub(crate) fn item_import(
        &mut self,
        from_namespace: Option<&Namespace>,
        path: Vec<Ident>,
        item: &Ident,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let base_namespace = from_namespace.unwrap_or(self);
        let namespace = check!(
            base_namespace.find_module_relative(&path),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut impls_to_insert = vec![];

        match namespace.symbols.get(item).cloned() {
            Some(decl) => {
                if decl.visibility() != Visibility::Public {
                    errors.push(CompileError::ImportPrivateSymbol {
                        name: item.as_str().to_string(),
                        span: item.span().clone(),
                    });
                }
                // if this is a const, insert it into the local namespace directly
                if let TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    is_mutable: VariableMutability::ExportedConst,
                    ref name,
                    ..
                }) = decl
                {
                    self.insert(alias.unwrap_or_else(|| name.clone()), decl.clone());
                    return ok((), warnings, errors);
                }
                let a = decl.return_type().value;
                //  if this is an enum or struct, import its implementations
                let mut res = match a {
                    Some(a) => namespace
                        .implemented_traits
                        .get_call_path_and_type_info(look_up_type_id(a)),
                    None => vec![],
                };
                impls_to_insert.append(&mut res);
                // no matter what, import it this way though.
                match alias.clone() {
                    Some(alias) => {
                        if self.use_synonyms.contains_key(&alias) {
                            errors.push(CompileError::ShadowsOtherSymbol {
                                name: alias.as_str().to_string(),
                                span: alias.span().clone(),
                            });
                        }
                        self.use_synonyms.insert(alias.clone(), path.clone());
                        self.use_aliases.insert(alias.as_str().to_string(), item.clone());
                    }
                    None => {
                        if self.use_synonyms.contains_key(item) {
                            errors.push(CompileError::ShadowsOtherSymbol {
                                name: item.as_str().to_string(),
                                span: item.span().clone(),
                            });
                        }
                        self.use_synonyms.insert(item.clone(), path.clone());
                    }
                };
            }
            None => {
                errors.push(CompileError::SymbolNotFound {
                    name: item.as_str().to_string(),
                    span: item.span().clone(),
                });
                return err(warnings, errors);
            }
        };

        impls_to_insert
            .into_iter()
            .for_each(|((call_path, type_info), methods)| {
                self.implemented_traits.insert(call_path, type_info, methods);
            });

        ok((), warnings, errors)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TraitMap {
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
    trait_map: Vec<(
        (TraitName, TypeInfo),
        HashMap<String, TypedFunctionDeclaration>,
    )>,
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
        let mut methods_map = HashMap::new();
        for method in methods.into_iter() {
            let method_name = method.name.as_str().to_string();
            methods_map.insert(method_name, method);
        }
        self.trait_map
            .push(((trait_name, type_implementing_for), methods_map));
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
