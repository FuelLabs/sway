use core::fmt::Write;
use hashbrown::hash_map::RawEntryMut;
use hashbrown::HashMap;
use std::sync::RwLock;
use sway_types::Ident;

use crate::concurrent_slab::ListDisplay;
use crate::error::{err, ok};
use crate::{
    concurrent_slab::ConcurrentSlab, decl_engine::*, engine_threading::*, error::*, language::ty,
    namespace::Path, type_system::priv_prelude::*, Namespace,
};

use sway_error::{error::CompileError, type_error::TypeError, warning::CompileWarning};
use sway_types::{span::Span, Spanned};

#[derive(Debug, Default)]
pub struct TypeEngine {
    pub(super) slab: ConcurrentSlab<TypeInfo>,
    storage_only_types: ConcurrentSlab<TypeInfo>,
    id_map: RwLock<HashMap<TypeInfo, TypeId>>,
}

impl TypeEngine {
    /// Inserts a [TypeInfo] into the [TypeEngine] and returns a [TypeId]
    /// referring to that [TypeInfo].
    pub(crate) fn insert(&self, decl_engine: &DeclEngine, ty: TypeInfo) -> TypeId {
        let mut id_map = self.id_map.write().unwrap();

        let engines = Engines::new(self, decl_engine);
        let hash_builder = id_map.hasher().clone();
        let ty_hash = make_hasher(&hash_builder, engines)(&ty);

        let raw_entry = id_map
            .raw_entry_mut()
            .from_hash(ty_hash, |x| x.eq(&ty, engines));
        match raw_entry {
            RawEntryMut::Occupied(o) => return *o.get(),
            RawEntryMut::Vacant(_) if ty.can_change(decl_engine) => {
                TypeId::new(self.slab.insert(ty))
            }
            RawEntryMut::Vacant(v) => {
                let type_id = TypeId::new(self.slab.insert(ty.clone()));
                v.insert_with_hasher(ty_hash, ty, type_id, make_hasher(&hash_builder, engines));
                type_id
            }
        }
    }

    /// Performs a lookup of `id` into the [TypeEngine].
    pub fn get(&self, id: TypeId) -> TypeInfo {
        self.slab.get(id.index())
    }

    /// Performs a lookup of `id` into the [TypeEngine] recursing when finding a
    /// [TypeInfo::Alias].
    pub fn get_unaliased(&self, id: TypeId) -> TypeInfo {
        // A slight infinite loop concern if we somehow have self-referential aliases, but that
        // shouldn't be possible.
        match self.slab.get(id.index()) {
            TypeInfo::Alias { ty, .. } => self.get_unaliased(ty.type_id),
            ty_info => ty_info,
        }
    }

    /// Denotes the given [TypeId] as being used with storage.
    pub(crate) fn set_type_as_storage_only(&self, id: TypeId) {
        self.storage_only_types.insert(self.get(id));
    }

    /// Checks if the given [TypeInfo] is a storage only type.
    pub(crate) fn is_type_info_storage_only(
        &self,
        decl_engine: &DeclEngine,
        ti: &TypeInfo,
    ) -> bool {
        self.storage_only_types
            .exists(|x| ti.is_subset_of(x, Engines::new(self, decl_engine)))
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a generic type and `received`
    /// is not).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn unify(
        &self,
        decl_engine: &DeclEngine,
        received: TypeId,
        expected: TypeId,
        type_subst_stack_top: &SubstList,
        span: &Span,
        help_text: &str,
        err_override: Option<CompileError>,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        let engines = Engines::new(self, decl_engine);
        if !UnifyCheck::new(engines, type_subst_stack_top).check(received, expected) {
            // create a "mismatched type" error unless the `err_override`
            // argument has been provided
            let mut errors = vec![];
            match err_override {
                Some(err_override) => {
                    errors.push(err_override);
                }
                None => {
                    errors.push(CompileError::TypeError(TypeError::MismatchedType {
                        expected: engines.help_out(expected).to_string(),
                        received: engines.help_out(received).to_string(),
                        help_text: help_text.to_string(),
                        span: span.clone(),
                    }));
                }
            }
            return (vec![], errors);
        }
        let (warnings, errors) = normalize_err(
            Unifier::new(engines, type_subst_stack_top, help_text).unify(received, expected, span),
        );
        if errors.is_empty() {
            (warnings, errors)
        } else if err_override.is_some() {
            // return the errors from unification unless the `err_override`
            // argument has been provided
            (warnings, vec![err_override.unwrap()])
        } else {
            (warnings, errors)
        }
    }

    pub(crate) fn to_typeinfo(&self, id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
        match self.get(id) {
            TypeInfo::Unknown => Err(TypeError::UnknownType {
                span: error_span.clone(),
            }),
            ty => Ok(ty),
        }
    }

    pub(crate) fn compare_subst_list_and_args(
        &self,
        subst_list: &SubstList,
        type_args: &[TypeArgument],
        enforce_type_args: EnforceTypeArguments,
        obj_name: &Ident,
        call_site_span: &Span,
    ) -> CompileResult<()> {
        let warnings = vec![];
        let mut errors = vec![];

        let type_arguments_span = type_args
            .iter()
            .map(|x| x.span.clone())
            .reduce(Span::join)
            .unwrap_or_else(|| obj_name.span());

        match (subst_list.len(), type_args.len()) {
            (n, 0) if n > 0 && enforce_type_args.is_yes() => {
                errors.push(CompileError::NeedsTypeArguments {
                    name: obj_name.clone(),
                    span: call_site_span.clone(),
                });
                return err(warnings, errors);
            }
            (0, m) if m > 0 => {
                errors.push(CompileError::DoesNotTakeTypeArguments {
                    name: obj_name.clone(),
                    span: type_arguments_span,
                });
                return err(warnings, errors);
            }
            (n, m) if n != m => {
                errors.push(CompileError::IncorrectNumberOfTypeArguments {
                    given: type_args.len(),
                    expected: subst_list.len(),
                    span: type_arguments_span,
                });
                return err(warnings, errors);
            }
            _ => {}
        }

        ok((), warnings, errors)
    }

    pub(crate) fn resolve_type_args(
        &self,
        namespace: &mut Namespace,
        decl_engine: &DeclEngine,
        mod_path: &Path,
        type_args: &mut [TypeArgument],
        enforce_type_args: EnforceTypeArguments,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        for type_arg in type_args.iter_mut() {
            type_arg.type_id = check!(
                self.resolve(
                    decl_engine,
                    type_arg.type_id,
                    &type_arg.span,
                    enforce_type_args,
                    None,
                    namespace,
                    mod_path
                ),
                self.insert(decl_engine, TypeInfo::ErrorRecovery),
                warnings,
                errors
            );
        }

        if errors.is_empty() {
            ok((), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn combine_subst_list_and_args<T>(
        &self,
        namespace: &mut Namespace,
        decl_engine: &DeclEngine,
        mod_path: &Path,
        decl_ref: &mut DeclRef<DeclId<T>>,
        type_args: &mut [TypeArgument],
        enforce_type_args: EnforceTypeArguments,
        call_site_span: &Span,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        check!(
            self.resolve_type_args(
                namespace,
                decl_engine,
                mod_path,
                type_args,
                enforce_type_args
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        check!(
            self.compare_subst_list_and_args(
                decl_ref.subst_list(),
                type_args,
                enforce_type_args,
                decl_ref.name(),
                call_site_span
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        decl_ref.subst_list_mut().apply_type_args(type_args);
        ok((), warnings, errors)
    }

    /// Resolve the type of the given [TypeId], replacing any instances of
    /// [TypeInfo::Custom] with either a monomorphized struct, monomorphized
    /// enum, or a reference to a type parameter.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve(
        &self,
        decl_engine: &DeclEngine,
        type_id: TypeId,
        call_site_span: &Span,
        enforce_type_args: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
        namespace: &mut Namespace,
        mod_path: &Path,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let engines = Engines::new(self, decl_engine);
        let module_path = type_info_prefix.unwrap_or(mod_path);
        let type_id = match self.get(type_id) {
            TypeInfo::Custom {
                call_path,
                type_arguments,
            } => {
                let mut type_args = type_arguments.unwrap_or_default();
                match namespace
                    .root()
                    .resolve_call_path_with_visibility_check(engines, module_path, &call_path)
                    .ok(&mut warnings, &mut errors)
                    .cloned()
                {
                    Some(ty::TyDecl::StructDecl {
                        name,
                        decl_id,
                        subst_list,
                        decl_span,
                    }) => {
                        let mut struct_ref =
                            DeclRef::new(name, decl_id, subst_list.scoped_copy(engines), decl_span);
                        check!(
                            self.combine_subst_list_and_args(
                                namespace,
                                decl_engine,
                                mod_path,
                                &mut struct_ref,
                                &mut type_args,
                                enforce_type_args,
                                call_site_span
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let type_id = self.insert(decl_engine, TypeInfo::Struct(struct_ref));
                        namespace.insert_trait_implementation_for_type(engines, type_id);
                        type_id
                    }
                    Some(ty::TyDecl::EnumDecl {
                        name,
                        decl_id,
                        subst_list,
                        decl_span,
                    }) => {
                        let mut enum_ref =
                            DeclRef::new(name, decl_id, subst_list.scoped_copy(engines), decl_span);
                        check!(
                            self.combine_subst_list_and_args(
                                namespace,
                                decl_engine,
                                mod_path,
                                &mut enum_ref,
                                &mut type_args,
                                enforce_type_args,
                                call_site_span
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let type_id = self.insert(decl_engine, TypeInfo::Enum(enum_ref));
                        namespace.insert_trait_implementation_for_type(engines, type_id);
                        type_id
                    }
                    Some(ty::TyDecl::TypeAliasDecl {
                        decl_id: original_id,
                        ..
                    }) => {
                        let new_copy = decl_engine.get_type_alias(&original_id);

                        // TODO: monomorphize the copy, in place, when generic type aliases are
                        // supported

                        let type_id = new_copy.create_type_id(engines);
                        namespace.insert_trait_implementation_for_type(engines, type_id);

                        type_id
                    }
                    Some(ty::TyDecl::GenericTypeForFunctionScope { type_id, .. }) => type_id,
                    _ => {
                        errors.push(CompileError::UnknownTypeName {
                            name: call_path.to_string(),
                            span: call_path.span(),
                        });
                        self.insert(decl_engine, TypeInfo::ErrorRecovery)
                    }
                }
            }
            TypeInfo::Array(mut elem_ty, n) => {
                elem_ty.type_id = check!(
                    self.resolve(
                        decl_engine,
                        elem_ty.type_id,
                        call_site_span,
                        enforce_type_args,
                        None,
                        namespace,
                        mod_path
                    ),
                    self.insert(decl_engine, TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );
                self.insert(decl_engine, TypeInfo::Array(elem_ty, n))
            }
            TypeInfo::Tuple(mut type_arguments) => {
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = check!(
                        self.resolve(
                            decl_engine,
                            type_argument.type_id,
                            call_site_span,
                            enforce_type_args,
                            None,
                            namespace,
                            mod_path
                        ),
                        self.insert(decl_engine, TypeInfo::ErrorRecovery),
                        warnings,
                        errors
                    );
                }
                self.insert(decl_engine, TypeInfo::Tuple(type_arguments))
            }
            _ => type_id,
        };
        ok(type_id, warnings, errors)
    }

    /// Pretty print method for printing the [TypeEngine]. This method is
    /// manually implemented to avoid implementation overhead regarding using
    /// [DisplayWithEngines].
    pub fn pretty_print(&self, decl_engine: &DeclEngine) -> String {
        let engines = Engines::new(self, decl_engine);
        let mut builder = String::new();
        self.slab.with_slice(|elems| {
            let list = elems
                .iter()
                .map(|type_info| format!("{:?}", engines.help_out(type_info)));
            let list = ListDisplay { list };
            write!(builder, "TypeEngine {{\n{list}\n}}").unwrap();
        });
        builder
    }
}

fn normalize_err(
    (w, e): (Vec<CompileWarning>, Vec<TypeError>),
) -> (Vec<CompileWarning>, Vec<CompileError>) {
    (w, e.into_iter().map(CompileError::from).collect())
}

/// This type is used to denote if, during monomorphization, the compiler
/// should enforce that type arguments be provided. An example of that
/// might be this:
///
/// ```ignore
/// struct Point<T> {
///   x: u64,
///   y: u64
/// }
///
/// fn add<T>(p1: Point<T>, p2: Point<T>) -> Point<T> {
///   Point {
///     x: p1.x + p2.x,
///     y: p1.y + p2.y
///   }
/// }
/// ```
///
/// `EnforeTypeArguments` would require that the type annotations
/// for `p1` and `p2` contain `<...>`. This is to avoid ambiguous definitions:
///
/// ```ignore
/// fn add(p1: Point, p2: Point) -> Point {
///   Point {
///     x: p1.x + p2.x,
///     y: p1.y + p2.y
///   }
/// }
/// ```
#[derive(Clone, Copy)]
pub(crate) enum EnforceTypeArguments {
    Yes,
    No,
}

impl EnforceTypeArguments {
    pub(crate) fn is_yes(&self) -> bool {
        matches!(self, EnforceTypeArguments::Yes)
    }
}
