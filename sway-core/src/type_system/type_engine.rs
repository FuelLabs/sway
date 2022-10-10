use super::*;
use crate::concurrent_slab::ConcurrentSlab;
use crate::declaration_engine::{
    de_add_monomorphized_enum_copy, de_add_monomorphized_struct_copy, de_get_enum, de_get_struct,
};
use crate::namespace::{Path, Root};
use crate::TyDeclaration;
use lazy_static::lazy_static;
use sway_error::error::CompileError;
use sway_error::type_error::TypeError;
use sway_types::span::Span;
use sway_types::{Ident, Spanned};

lazy_static! {
    static ref TYPE_ENGINE: TypeEngine = TypeEngine::default();
}

#[derive(Debug, Default)]
pub(crate) struct TypeEngine {
    slab: ConcurrentSlab<TypeInfo>,
    storage_only_types: ConcurrentSlab<TypeInfo>,
}

impl TypeEngine {
    /// Inserts a [TypeInfo] into the [TypeEngine] and returns a [TypeId]
    /// referring to that [TypeInfo].
    pub(crate) fn insert_type(&self, ty: TypeInfo) -> TypeId {
        TypeId::new(self.slab.insert(ty))
    }

    /// Gets the size of the [TypeEngine].
    fn size(&self) -> usize {
        self.slab.size()
    }

    /// Performs a lookup of `id` into the [TypeEngine].
    pub(crate) fn look_up_type_id(&self, id: TypeId) -> TypeInfo {
        self.slab.get(*id)
    }

    /// Denotes the given [TypeId] as being used with storage.
    fn set_type_as_storage_only(&self, id: TypeId) {
        self.storage_only_types.insert(self.look_up_type_id(id));
    }

    /// Checks if the given [TypeId] is a storage only type.
    fn is_type_storage_only(&self, id: TypeId) -> bool {
        let ti = &self.look_up_type_id(id);
        self.is_type_info_storage_only(ti)
    }

    /// Checks if the given [TypeInfo] is a storage only type.
    fn is_type_info_storage_only(&self, ti: &TypeInfo) -> bool {
        self.storage_only_types.exists(|x| ti.is_subset_of(x))
    }

    /// Given a `value` of type `T` that is able to be monomorphized and a set
    /// of `type_arguments`, monomorphize `value` with the `type_arguments`.
    ///
    /// When this function is called, it is passed a `T` that is a copy of some
    /// original declaration for `T` (let's denote the original with `[T]`).
    /// Because monomorphization happens at application time (e.g. function
    /// application), we want to be able to modify `value` such that type
    /// checking the application of `value` affects only `T` and not `[T]`.
    ///
    /// So, at a high level, this function does two things. It 1) performs the
    /// necessary work to refresh the relevant generic types in `T` so that they
    /// are distinct from the generics of the same name in `[T]`. And it 2)
    /// applies `type_arguments` (if any are provided) to the type parameters
    /// of `value`, unifying the types.
    ///
    /// There are 4 cases that are handled in this function:
    ///
    /// 1. `value` does not have type parameters + `type_arguments` is empty:
    ///     1a. return ok
    /// 2. `value` has type parameters + `type_arguments` is empty:
    ///     2a. if the [EnforceTypeArguments::Yes] variant is provided, then
    ///         error
    ///     2b. refresh the generic types with a [TypeMapping]
    /// 3. `value` does have type parameters + `type_arguments` is nonempty:
    ///     3a. error
    /// 4. `value` has type parameters + `type_arguments` is nonempty:
    ///     4a. check to see that the type parameters and `type_arguments` have
    ///         the same length
    ///     4b. for each type argument in `type_arguments`, resolve the type
    ///     4c. refresh the generic types with a [TypeMapping]
    fn monomorphize<T>(
        &self,
        value: &mut T,
        type_arguments: &mut [TypeArgument],
        enforce_type_arguments: EnforceTypeArguments,
        call_site_span: &Span,
        namespace: &Root,
        mod_path: &Path,
    ) -> CompileResult<()>
    where
        T: MonomorphizeHelper + CopyTypes,
    {
        let mut warnings = vec![];
        let mut errors = vec![];
        match (
            value.type_parameters().is_empty(),
            type_arguments.is_empty(),
        ) {
            (true, true) => ok((), warnings, errors),
            (false, true) => {
                if let EnforceTypeArguments::Yes = enforce_type_arguments {
                    errors.push(CompileError::NeedsTypeArguments {
                        name: value.name().clone(),
                        span: call_site_span.clone(),
                    });
                    return err(warnings, errors);
                }
                let type_mapping = TypeMapping::from_type_parameters(value.type_parameters());
                value.copy_types(&type_mapping);
                ok((), warnings, errors)
            }
            (true, false) => {
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| value.name().span());
                errors.push(CompileError::DoesNotTakeTypeArguments {
                    name: value.name().clone(),
                    span: type_arguments_span,
                });
                err(warnings, errors)
            }
            (false, false) => {
                let type_arguments_span = type_arguments
                    .iter()
                    .map(|x| x.span.clone())
                    .reduce(Span::join)
                    .unwrap_or_else(|| value.name().span());
                if value.type_parameters().len() != type_arguments.len() {
                    errors.push(CompileError::IncorrectNumberOfTypeArguments {
                        given: type_arguments.len(),
                        expected: value.type_parameters().len(),
                        span: type_arguments_span,
                    });
                    return err(warnings, errors);
                }
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = check!(
                        self.resolve_type(
                            type_argument.type_id,
                            &type_argument.span,
                            enforce_type_arguments,
                            None,
                            namespace,
                            mod_path
                        ),
                        self.insert_type(TypeInfo::ErrorRecovery),
                        warnings,
                        errors
                    );
                }
                let type_mapping = TypeMapping::from_type_parameters(value.type_parameters());
                check!(
                    type_mapping.unify_with_type_arguments(type_arguments),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                value.copy_types(&type_mapping);
                ok((), warnings, errors)
            }
        }
    }

    /// Make the types of `received` and `expected` equivalent (or produce an
    /// error if there is a conflict between them).
    ///
    /// More specifically, this function tries to make `received` equivalent to
    /// `expected`, except in cases where `received` has more type information
    /// than `expected` (e.g. when `expected` is a generic type and `received`
    /// is not).
    pub(crate) fn unify(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        use TypeInfo::*;

        // a curried version of this method to use in the helper functions
        let curried = |received: TypeId, expected: TypeId, span: &Span, help_text: &str| {
            self.unify(received, expected, span, help_text)
        };

        match (self.slab.get(*received), self.slab.get(*expected)) {
            // If they have the same `TypeInfo`, then we either compare them for
            // correctness or perform further unification.
            (Boolean, Boolean) => (vec![], vec![]),
            (SelfType, SelfType) => (vec![], vec![]),
            (B256, B256) => (vec![], vec![]),
            (Numeric, Numeric) => (vec![], vec![]),
            (Contract, Contract) => (vec![], vec![]),
            (Str(l), Str(r)) => unify::unify_strs(received, expected, span, help_text, l, r),
            (Tuple(rfs), Tuple(efs)) if rfs.len() == efs.len() => {
                unify::unify_tuples(help_text, rfs, efs, curried)
            }
            (UnsignedInteger(r), UnsignedInteger(e)) => unify::unify_unsigned_ints(span, r, e),
            (Numeric, e @ UnsignedInteger(_)) => match self.slab.replace(received, &Numeric, e) {
                None => (vec![], vec![]),
                Some(_) => self.unify(received, expected, span, help_text),
            },
            (r @ UnsignedInteger(_), Numeric) => match self.slab.replace(expected, &Numeric, r) {
                None => (vec![], vec![]),
                Some(_) => self.unify(received, expected, span, help_text),
            },
            (
                Struct {
                    name: rn,
                    type_parameters: rpts,
                    fields: rfs,
                },
                Struct {
                    name: en,
                    type_parameters: etps,
                    fields: efs,
                },
            ) => unify::unify_structs(
                received,
                expected,
                span,
                help_text,
                (rn, rpts, rfs),
                (en, etps, efs),
                curried,
            ),
            (
                Enum {
                    name: rn,
                    type_parameters: rtps,
                    variant_types: rvs,
                },
                Enum {
                    name: en,
                    type_parameters: etps,
                    variant_types: evs,
                },
            ) => unify::unify_enums(
                received,
                expected,
                span,
                help_text,
                (rn, rtps, rvs),
                (en, etps, evs),
                curried,
            ),
            (Array(re, rc, _), Array(ee, ec, _)) if rc == ec => {
                unify::unify_arrays(received, expected, span, help_text, re, ee, curried)
            }
            (
                ref r @ TypeInfo::ContractCaller {
                    abi_name: ref ran,
                    address: ref rra,
                },
                TypeInfo::ContractCaller {
                    abi_name: ref ean, ..
                },
            ) if (ran == ean && rra.is_none()) || matches!(ran, AbiName::Deferred) => {
                // if one address is empty, coerce to the other one
                match self.slab.replace(received, r, look_up_type_id(expected)) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (
                TypeInfo::ContractCaller {
                    abi_name: ref ran, ..
                },
                ref e @ TypeInfo::ContractCaller {
                    abi_name: ref ean,
                    address: ref ea,
                },
            ) if (ran == ean && ea.is_none()) || matches!(ean, AbiName::Deferred) => {
                // if one address is empty, coerce to the other one
                match self.slab.replace(expected, e, look_up_type_id(received)) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (ref r @ TypeInfo::ContractCaller { .. }, ref e @ TypeInfo::ContractCaller { .. })
                if r == e =>
            {
                // if they are the same, then it's ok
                (vec![], vec![])
            }

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, Unknown) => (vec![], vec![]),
            (Unknown, e) => match self.slab.replace(received, &Unknown, e) {
                None => (vec![], vec![]),
                Some(_) => self.unify(received, expected, span, help_text),
            },
            (r, Unknown) => match self.slab.replace(expected, &Unknown, r) {
                None => (vec![], vec![]),
                Some(_) => self.unify(received, expected, span, help_text),
            },

            (UnknownGeneric { name: rn }, UnknownGeneric { name: en })
                if rn.as_str() == en.as_str() =>
            {
                (vec![], vec![])
            }
            (ref r @ UnknownGeneric { .. }, e) => match self.slab.replace(received, r, e) {
                None => (vec![], vec![]),
                Some(_) => self.unify(received, expected, span, help_text),
            },
            (r, ref e @ UnknownGeneric { .. }) => match self.slab.replace(expected, e, r) {
                None => (vec![], vec![]),
                Some(_) => self.unify(received, expected, span, help_text),
            },

            // If no previous attempts to unify were successful, raise an error
            (TypeInfo::ErrorRecovery, _) => (vec![], vec![]),
            (_, TypeInfo::ErrorRecovery) => (vec![], vec![]),
            (r, e) => {
                let errors = vec![TypeError::MismatchedType {
                    expected: r.to_string(),
                    received: e.to_string(),
                    help_text: help_text.to_string(),
                    span: span.clone(),
                }];
                (vec![], errors)
            }
        }
    }

    /// Make the type of `expected` equivalent to `received`.
    pub(crate) fn unify_right(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        use TypeInfo::*;

        // a curried version of this method to use in the helper functions
        let curried = |received: TypeId, expected: TypeId, span: &Span, help_text: &str| {
            self.unify_right(received, expected, span, help_text)
        };

        match (self.slab.get(*received), self.slab.get(*expected)) {
            // If they have the same `TypeInfo`, then we either compare them for
            // correctness or perform further unification.
            (Boolean, Boolean) => (vec![], vec![]),
            (SelfType, SelfType) => (vec![], vec![]),
            (B256, B256) => (vec![], vec![]),
            (Numeric, Numeric) => (vec![], vec![]),
            (Contract, Contract) => (vec![], vec![]),
            (Str(l), Str(r)) => unify::unify_strs(received, expected, span, help_text, l, r),
            (Tuple(rfs), Tuple(efs)) if rfs.len() == efs.len() => {
                unify::unify_tuples(help_text, rfs, efs, curried)
            }
            (UnsignedInteger(r), UnsignedInteger(e)) => unify::unify_unsigned_ints(span, r, e),
            (Numeric, UnsignedInteger(_)) => (vec![], vec![]),
            (r @ UnsignedInteger(_), Numeric) => match self.slab.replace(expected, &Numeric, r) {
                None => (vec![], vec![]),
                Some(_) => self.unify_right(received, expected, span, help_text),
            },
            (
                Struct {
                    name: rn,
                    type_parameters: rpts,
                    fields: rfs,
                },
                Struct {
                    name: en,
                    type_parameters: etps,
                    fields: efs,
                },
            ) => unify::unify_structs(
                received,
                expected,
                span,
                help_text,
                (rn, rpts, rfs),
                (en, etps, efs),
                curried,
            ),
            (
                Enum {
                    name: rn,
                    type_parameters: rtps,
                    variant_types: rvs,
                },
                Enum {
                    name: en,
                    type_parameters: etps,
                    variant_types: evs,
                },
            ) => unify::unify_enums(
                received,
                expected,
                span,
                help_text,
                (rn, rtps, rvs),
                (en, etps, evs),
                curried,
            ),
            (Array(re, rc, _), Array(ee, ec, _)) if rc == ec => {
                unify::unify_arrays(received, expected, span, help_text, re, ee, curried)
            }
            (
                TypeInfo::ContractCaller {
                    abi_name: ref ran, ..
                },
                ref e @ TypeInfo::ContractCaller {
                    abi_name: ref ean,
                    address: ref ea,
                },
            ) if (ran == ean && ea.is_none()) || matches!(ean, AbiName::Deferred) => {
                // if one address is empty, coerce to the other one
                match self.slab.replace(expected, e, look_up_type_id(received)) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify_right(received, expected, span, help_text),
                }
            }
            (
                TypeInfo::ContractCaller {
                    abi_name: ref ran,
                    address: ref ra,
                },
                TypeInfo::ContractCaller {
                    abi_name: ref ean, ..
                },
            ) if (ran == ean && ra.is_none()) || matches!(ran, AbiName::Deferred) => {
                (vec![], vec![])
            }
            (ref r @ TypeInfo::ContractCaller { .. }, ref e @ TypeInfo::ContractCaller { .. })
                if r == e =>
            {
                // if they are the same, then it's ok
                (vec![], vec![])
            }

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, Unknown) => (vec![], vec![]),
            (r, Unknown) => match self.slab.replace(expected, &Unknown, r) {
                None => (vec![], vec![]),
                Some(_) => self.unify_right(received, expected, span, help_text),
            },
            (Unknown, _) => (vec![], vec![]),

            (UnknownGeneric { name: rn }, UnknownGeneric { name: en })
                if rn.as_str() == en.as_str() =>
            {
                (vec![], vec![])
            }
            (r, ref e @ UnknownGeneric { .. }) => match self.slab.replace(expected, e, r) {
                None => (vec![], vec![]),
                Some(_) => self.unify_right(received, expected, span, help_text),
            },
            // this case is purposefully removed because it should cause an
            // error. trying to unify_right a generic with anything other an an
            // unknown or another generic is a type error
            //(UnknownGeneric { .. }, _) => (vec![], vec![]),

            // If no previous attempts to unify were successful, raise an error
            (TypeInfo::ErrorRecovery, _) => (vec![], vec![]),
            (_, TypeInfo::ErrorRecovery) => (vec![], vec![]),
            (r, e) => {
                let errors = vec![TypeError::MismatchedType {
                    expected: r.to_string(),
                    received: e.to_string(),
                    help_text: help_text.to_string(),
                    span: span.clone(),
                }];
                (vec![], errors)
            }
        }
    }

    /// Replace any instances of the [TypeInfo::SelfType] variant with
    /// `self_type` in both `received` and `expected`, then unify `received` and
    /// `expected`.
    fn unify_with_self(
        &self,
        mut received: TypeId,
        mut expected: TypeId,
        self_type: TypeId,
        span: &Span,
        help_text: &str,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        received.replace_self_type(self_type);
        expected.replace_self_type(self_type);
        self.unify(received, expected, span, help_text)
    }

    pub fn to_typeinfo(&self, id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
        match self.look_up_type_id(id) {
            TypeInfo::Unknown => Err(TypeError::UnknownType {
                span: error_span.clone(),
            }),
            ty => Ok(ty),
        }
    }

    /// Clear the [TypeEngine].
    fn clear(&self) {
        self.slab.clear();
        self.storage_only_types.clear();
    }

    /// Resolve the type of the given [TypeId], replacing any instances of
    /// [TypeInfo::Custom] with either a monomorphized struct, monomorphized
    /// enum, or a reference to a type parameter.
    fn resolve_type(
        &self,
        type_id: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
        namespace: &Root,
        mod_path: &Path,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let module_path = type_info_prefix.unwrap_or(mod_path);
        let type_id = match look_up_type_id(type_id) {
            TypeInfo::Custom {
                name,
                type_arguments,
            } => {
                match namespace
                    .resolve_symbol(module_path, &name)
                    .ok(&mut warnings, &mut errors)
                    .cloned()
                {
                    Some(TyDeclaration::StructDeclaration(original_id)) => {
                        // get the copy from the declaration engine
                        let mut new_copy = check!(
                            CompileResult::from(de_get_struct(original_id.clone(), &name.span())),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );

                        // monomorphize the copy, in place
                        check!(
                            self.monomorphize(
                                &mut new_copy,
                                &mut type_arguments.unwrap_or_default(),
                                enforce_type_arguments,
                                span,
                                namespace,
                                mod_path
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors,
                        );

                        // create the type id from the copy
                        let type_id = new_copy.create_type_id();

                        // add the new copy as a monomorphized copy of the original id
                        de_add_monomorphized_struct_copy(original_id, new_copy);

                        // return the id
                        type_id
                    }
                    Some(TyDeclaration::EnumDeclaration(original_id)) => {
                        // get the copy from the declaration engine
                        let mut new_copy = check!(
                            CompileResult::from(de_get_enum(original_id.clone(), &name.span())),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );

                        // monomorphize the copy, in place
                        check!(
                            self.monomorphize(
                                &mut new_copy,
                                &mut type_arguments.unwrap_or_default(),
                                enforce_type_arguments,
                                span,
                                namespace,
                                mod_path
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );

                        // create the type id from the copy
                        let type_id = new_copy.create_type_id();

                        // add the new copy as a monomorphized copy of the original id
                        de_add_monomorphized_enum_copy(original_id, new_copy);

                        // return the id
                        type_id
                    }
                    Some(TyDeclaration::GenericTypeForFunctionScope { type_id, .. }) => type_id,
                    _ => {
                        errors.push(CompileError::UnknownTypeName {
                            name: name.to_string(),
                            span: name.span(),
                        });
                        self.insert_type(TypeInfo::ErrorRecovery)
                    }
                }
            }
            TypeInfo::Array(type_id, n, initial_type_id) => {
                let new_type_id = check!(
                    self.resolve_type(
                        type_id,
                        span,
                        enforce_type_arguments,
                        None,
                        namespace,
                        mod_path
                    ),
                    self.insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );
                self.insert_type(TypeInfo::Array(new_type_id, n, initial_type_id))
            }
            TypeInfo::Tuple(mut type_arguments) => {
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = check!(
                        self.resolve_type(
                            type_argument.type_id,
                            span,
                            enforce_type_arguments,
                            None,
                            namespace,
                            mod_path
                        ),
                        self.insert_type(TypeInfo::ErrorRecovery),
                        warnings,
                        errors
                    );
                }
                self.insert_type(TypeInfo::Tuple(type_arguments))
            }
            _ => type_id,
        };
        ok(type_id, warnings, errors)
    }

    /// Replace any instances of the [TypeInfo::SelfType] variant with
    /// `self_type` in `type_id`, then resolve `type_id`.
    #[allow(clippy::too_many_arguments)]
    fn resolve_type_with_self(
        &self,
        mut type_id: TypeId,
        self_type: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
        namespace: &Root,
        mod_path: &Path,
    ) -> CompileResult<TypeId> {
        type_id.replace_self_type(self_type);
        self.resolve_type(
            type_id,
            span,
            enforce_type_arguments,
            type_info_prefix,
            namespace,
            mod_path,
        )
    }
}

pub fn insert_type(ty: TypeInfo) -> TypeId {
    TYPE_ENGINE.insert_type(ty)
}

pub fn type_engine_size() -> usize {
    TYPE_ENGINE.size()
}

pub fn look_up_type_id(id: TypeId) -> TypeInfo {
    TYPE_ENGINE.look_up_type_id(id)
}

pub fn set_type_as_storage_only(id: TypeId) {
    TYPE_ENGINE.set_type_as_storage_only(id);
}

pub fn is_type_storage_only(id: TypeId) -> bool {
    TYPE_ENGINE.is_type_storage_only(id)
}

pub fn is_type_info_storage_only(ti: &TypeInfo) -> bool {
    TYPE_ENGINE.is_type_info_storage_only(ti)
}

pub(crate) fn monomorphize<T>(
    value: &mut T,
    type_arguments: &mut [TypeArgument],
    enforce_type_arguments: EnforceTypeArguments,
    call_site_span: &Span,
    namespace: &Root,
    module_path: &Path,
) -> CompileResult<()>
where
    T: MonomorphizeHelper + CopyTypes,
{
    TYPE_ENGINE.monomorphize(
        value,
        type_arguments,
        enforce_type_arguments,
        call_site_span,
        namespace,
        module_path,
    )
}

pub fn unify_with_self(
    received: TypeId,
    expected: TypeId,
    self_type: TypeId,
    span: &Span,
    help_text: &str,
) -> (Vec<CompileWarning>, Vec<CompileError>) {
    let (warnings, errors) =
        TYPE_ENGINE.unify_with_self(received, expected, self_type, span, help_text);
    (
        warnings,
        errors.into_iter().map(|error| error.into()).collect(),
    )
}

pub(crate) fn unify(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
) -> (Vec<CompileWarning>, Vec<CompileError>) {
    let (warnings, errors) = TYPE_ENGINE.unify(received, expected, span, help_text);
    (
        warnings,
        errors.into_iter().map(|error| error.into()).collect(),
    )
}

pub(crate) fn unify_right(
    received: TypeId,
    expected: TypeId,
    span: &Span,
    help_text: &str,
) -> (Vec<CompileWarning>, Vec<CompileError>) {
    let (warnings, errors) = TYPE_ENGINE.unify_right(received, expected, span, help_text);
    (
        warnings,
        errors.into_iter().map(|error| error.into()).collect(),
    )
}

pub(crate) fn to_typeinfo(id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
    TYPE_ENGINE.to_typeinfo(id, error_span)
}

pub fn clear_type_engine() {
    TYPE_ENGINE.clear();
}

pub(crate) fn resolve_type(
    type_id: TypeId,
    span: &Span,
    enforce_type_arguments: EnforceTypeArguments,
    type_info_prefix: Option<&Path>,
    namespace: &Root,
    mod_path: &Path,
) -> CompileResult<TypeId> {
    TYPE_ENGINE.resolve_type(
        type_id,
        span,
        enforce_type_arguments,
        type_info_prefix,
        namespace,
        mod_path,
    )
}

pub(crate) fn resolve_type_with_self(
    type_id: TypeId,
    self_type: TypeId,
    span: &Span,
    enforce_type_arguments: EnforceTypeArguments,
    type_info_prefix: Option<&Path>,
    namespace: &Root,
    mod_path: &Path,
) -> CompileResult<TypeId> {
    TYPE_ENGINE.resolve_type_with_self(
        type_id,
        self_type,
        span,
        enforce_type_arguments,
        type_info_prefix,
        namespace,
        mod_path,
    )
}

pub(crate) trait MonomorphizeHelper {
    fn name(&self) -> &Ident;
    fn type_parameters(&self) -> &[TypeParameter];
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
