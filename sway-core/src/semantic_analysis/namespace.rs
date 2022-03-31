use crate::{
    error::*, type_engine::*, CallPath, CompileResult, Ident, TypeArgument, TypeInfo,
    TypeParameter, TypedDeclaration, TypedFunctionDeclaration,
};

use crate::semantic_analysis::{
    ast_node::TypedStorageDeclaration, declaration::TypedStorageField, TypeCheckedStorageAccess,
};

use sway_types::span::Span;

use std::collections::{BTreeMap, HashMap};

pub mod arena;
pub use arena::*;

type ModuleName = String;
type TraitName = CallPath;
/// A namespace represents all items that exist either via declaration or importing.
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
    modules: BTreeMap<ModuleName, NamespaceRef>,
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

    pub fn get_all_imported_modules(&self) -> impl Iterator<Item = &NamespaceRef> {
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
                    errors.push(CompileError::ShadowsOtherSymbol {
                        span: name.span().clone(),
                        name: name.as_str().to_string(),
                    });
                }
                TypedDeclaration::GenericTypeForFunctionScope { .. } => {
                    errors.push(CompileError::GenericShadowsGeneric {
                        span: name.span().clone(),
                        name: name.as_str().to_string(),
                    });
                }
                _ => {
                    warnings.push(CompileWarning {
                        span: name.span().clone(),
                        warning_content: Warning::ShadowsOtherSymbol {
                            name: name.span().as_str().to_string(),
                        },
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

    pub fn insert_module(&mut self, module_name: String, ix: NamespaceRef) {
        self.modules.insert(module_name, ix);
    }

    pub fn insert_dependency_module(&mut self, module_name: String, ix: NamespaceRef) {
        self.insert_module(module_name, ix)
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
