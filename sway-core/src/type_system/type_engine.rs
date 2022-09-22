use super::*;
use crate::concurrent_slab::ConcurrentSlab;
use crate::declaration_engine::{
    de_add_monomorphized_enum_copy, de_add_monomorphized_struct_copy, de_get_enum, de_get_struct,
};
use crate::namespace::{Path, Root};
use crate::TypedDeclaration;
use lazy_static::lazy_static;
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
    pub fn insert_type(&self, ty: TypeInfo) -> TypeId {
        TypeId::new(self.slab.insert(ty))
    }

    pub fn look_up_type_id_raw(&self, id: TypeId) -> TypeInfo {
        self.slab.get(*id)
    }

    pub fn look_up_type_id(&self, id: TypeId) -> TypeInfo {
        match self.slab.get(*id) {
            TypeInfo::Ref(other, _sp) => self.look_up_type_id(other),
            ty => ty,
        }
    }

    pub fn set_type_as_storage_only(&self, id: TypeId) {
        self.storage_only_types.insert(self.look_up_type_id(id));
    }

    pub fn is_type_storage_only(&self, id: TypeId) -> bool {
        let ti = &self.look_up_type_id(id);
        self.is_type_info_storage_only(ti)
    }

    pub fn is_type_info_storage_only(&self, ti: &TypeInfo) -> bool {
        self.storage_only_types.exists(|x| ti.is_subset_of(x))
    }

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
                let type_mapping = insert_type_parameters(value.type_parameters());
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
                let type_mapping = insert_type_parameters(value.type_parameters());
                for ((_, interim_type), type_argument) in
                    type_mapping.iter().zip(type_arguments.iter())
                {
                    let (mut new_warnings, new_errors) = unify(
                        *interim_type,
                        type_argument.type_id,
                        &type_argument.span,
                        "Type argument is not assignable to generic type parameter.",
                    );
                    warnings.append(&mut new_warnings);
                    errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
                }
                value.copy_types(&type_mapping);
                ok((), warnings, errors)
            }
        }
    }

    /// Make the types of two type terms equivalent (or produce an error if
    /// there is a conflict between them).
    //
    // When reporting type errors we will report 'received' and 'expected' as such.
    pub(crate) fn unify(
        &self,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: impl Into<String>,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        use TypeInfo::*;
        let help_text = help_text.into();
        match (self.slab.get(*received), self.slab.get(*expected)) {
            // If the types are exactly the same, we are done.
            (Boolean, Boolean) => (vec![], vec![]),
            (SelfType, SelfType) => (vec![], vec![]),
            (Byte, Byte) => (vec![], vec![]),
            (B256, B256) => (vec![], vec![]),
            (Numeric, Numeric) => (vec![], vec![]),
            (Contract, Contract) => (vec![], vec![]),
            (Str(l), Str(r)) => {
                let warnings = vec![];
                let mut errors = vec![];
                if l != r {
                    errors.push(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }
            //(received_info, expected_info) if received_info == expected_info => (vec![], vec![]),

            // Follow any references
            (Ref(received, _sp1), Ref(expected, _sp2)) if received == expected => (vec![], vec![]),
            (Ref(received, _sp), _) => self.unify(received, expected, span, help_text),
            (_, Ref(expected, _sp)) => self.unify(received, expected, span, help_text),

            // When we don't know anything about either term, assume that
            // they match and make the one we know nothing about reference the
            // one we may know something about
            (Unknown, Unknown) => (vec![], vec![]),
            (Unknown, _) => {
                match self
                    .slab
                    .replace(received, &Unknown, TypeInfo::Ref(expected, span.clone()))
                {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (_, Unknown) => {
                match self
                    .slab
                    .replace(expected, &Unknown, TypeInfo::Ref(received, span.clone()))
                {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }

            (Tuple(fields_a), Tuple(fields_b)) if fields_a.len() == fields_b.len() => {
                let mut warnings = vec![];
                let mut errors = vec![];
                for (field_a, field_b) in fields_a.iter().zip(fields_b.iter()) {
                    let (new_warnings, new_errors) = self.unify(
                        field_a.type_id,
                        field_b.type_id,
                        &field_a.span,
                        help_text.clone(),
                    );
                    warnings.extend(new_warnings);
                    errors.extend(new_errors);
                }
                (warnings, errors)
            }

            (UnsignedInteger(received_width), UnsignedInteger(expected_width)) => {
                // E.g., in a variable declaration `let a: u32 = 10u64` the 'expected' type will be
                // the annotation `u32`, and the 'received' type is 'self' of the initialiser, or
                // `u64`.  So we're casting received TO expected.
                let warnings = match numeric_cast_compat(expected_width, received_width) {
                    NumericCastCompatResult::CastableWithWarning(warn) => {
                        vec![CompileWarning {
                            span: span.clone(),
                            warning_content: warn,
                        }]
                    }
                    NumericCastCompatResult::Compatible => {
                        vec![]
                    }
                };

                // we don't want to do a slab replacement here, because
                // we don't want to overwrite the original numeric type with the new one.
                // This isn't actually inferencing the original type to the new numeric type.
                // We just want to say "up until this point, this was a u32 (eg) and now it is a
                // u64 (eg)". If we were to do a slab replace here, we'd be saying "this was always a
                // u64 (eg)".
                (warnings, vec![])
            }

            (UnknownGeneric { name: l_name }, UnknownGeneric { name: r_name })
                if l_name.as_str() == r_name.as_str() =>
            {
                (vec![], vec![])
            }
            (ref received_info @ UnknownGeneric { .. }, _) => {
                self.slab.replace(
                    received,
                    received_info,
                    TypeInfo::Ref(expected, span.clone()),
                );
                (vec![], vec![])
            }

            (_, ref expected_info @ UnknownGeneric { .. }) => {
                self.slab.replace(
                    expected,
                    expected_info,
                    TypeInfo::Ref(received, span.clone()),
                );
                (vec![], vec![])
            }

            // if the types, once their ids have been looked up, are the same, we are done
            (
                Struct {
                    name: a_name,
                    fields: a_fields,
                    type_parameters: a_parameters,
                    ..
                },
                Struct {
                    name: b_name,
                    fields: b_fields,
                    type_parameters: b_parameters,
                    ..
                },
            ) => {
                let mut warnings = vec![];
                let mut errors = vec![];
                if a_name == b_name
                    && a_fields.len() == b_fields.len()
                    && a_parameters.len() == b_parameters.len()
                {
                    a_fields.iter().zip(b_fields.iter()).for_each(|(a, b)| {
                        let (new_warnings, new_errors) =
                            self.unify(a.type_id, b.type_id, &a.span, help_text.clone());
                        warnings.extend(new_warnings);
                        errors.extend(new_errors);
                    });
                    a_parameters
                        .iter()
                        .zip(b_parameters.iter())
                        .for_each(|(a, b)| {
                            let (new_warnings, new_errors) = self.unify(
                                a.type_id,
                                b.type_id,
                                &a.name_ident.span(),
                                help_text.clone(),
                            );
                            warnings.extend(new_warnings);
                            errors.extend(new_errors);
                        });
                } else {
                    errors.push(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }
            (
                Enum {
                    name: a_name,
                    variant_types: a_variants,
                    type_parameters: a_parameters,
                },
                Enum {
                    name: b_name,
                    variant_types: b_variants,
                    type_parameters: b_parameters,
                },
            ) => {
                let mut warnings = vec![];
                let mut errors = vec![];
                if a_name == b_name
                    && a_variants.len() == b_variants.len()
                    && a_parameters.len() == b_parameters.len()
                {
                    a_variants.iter().zip(b_variants.iter()).for_each(|(a, b)| {
                        let (new_warnings, new_errors) =
                            self.unify(a.type_id, b.type_id, &a.span, help_text.clone());
                        warnings.extend(new_warnings);
                        errors.extend(new_errors);
                    });
                    a_parameters
                        .iter()
                        .zip(b_parameters.iter())
                        .for_each(|(a, b)| {
                            let (new_warnings, new_errors) = self.unify(
                                a.type_id,
                                b.type_id,
                                &a.name_ident.span(),
                                help_text.clone(),
                            );
                            warnings.extend(new_warnings);
                            errors.extend(new_errors);
                        });
                } else {
                    errors.push(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }

            (Numeric, expected_info @ UnsignedInteger(_)) => {
                match self.slab.replace(received, &Numeric, expected_info) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (received_info @ UnsignedInteger(_), Numeric) => {
                match self.slab.replace(expected, &Numeric, received_info) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }

            (Array(a_elem, a_count, _), Array(b_elem, b_count, _)) if a_count == b_count => {
                let (warnings, new_errors) = self.unify(a_elem, b_elem, span, help_text.clone());

                // If there was an error then we want to report the array types as mismatching, not
                // the elem types.
                let mut errors = vec![];
                if !new_errors.is_empty() {
                    errors.push(TypeError::MismatchedType {
                        expected,
                        received,
                        help_text,
                        span: span.clone(),
                    });
                }
                (warnings, errors)
            }

            (
                ref r @ TypeInfo::ContractCaller {
                    abi_name: ref abi_name_received,
                    address: ref received_address,
                },
                TypeInfo::ContractCaller {
                    abi_name: ref abi_name_expected,
                    ..
                },
            ) if (abi_name_received == abi_name_expected && received_address.is_none())
                || matches!(abi_name_received, AbiName::Deferred) =>
            {
                // if one address is empty, coerce to the other one
                match self.slab.replace(received, r, look_up_type_id(expected)) {
                    None => (vec![], vec![]),
                    Some(_) => self.unify(received, expected, span, help_text),
                }
            }
            (
                TypeInfo::ContractCaller {
                    abi_name: ref abi_name_received,
                    ..
                },
                ref e @ TypeInfo::ContractCaller {
                    abi_name: ref abi_name_expected,
                    ref address,
                },
            ) if (abi_name_received == abi_name_expected && address.is_none())
                || matches!(abi_name_expected, AbiName::Deferred) =>
            {
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
            // When unifying complex types, we must check their sub-types. This
            // can be trivially implemented for tuples, sum types, etc.
            // (List(a_item), List(b_item)) => self.unify(a_item, b_item),
            // this can be used for curried function types but we might not want that
            // (Func(a_i, a_o), Func(b_i, b_o)) => {
            //     self.unify(a_i, b_i).and_then(|_| self.unify(a_o, b_o))
            // }

            // If no previous attempts to unify were successful, raise an error
            (TypeInfo::ErrorRecovery, _) => (vec![], vec![]),
            (_, TypeInfo::ErrorRecovery) => (vec![], vec![]),
            (_, _) => {
                let errors = vec![TypeError::MismatchedType {
                    expected,
                    received,
                    help_text,
                    span: span.clone(),
                }];
                (vec![], errors)
            }
        }
    }

    pub fn unify_with_self(
        &self,
        mut received: TypeId,
        mut expected: TypeId,
        self_type: TypeId,
        span: &Span,
        help_text: impl Into<String>,
    ) -> (Vec<CompileWarning>, Vec<TypeError>) {
        received.replace_self_type(self_type);
        expected.replace_self_type(self_type);
        self.unify(received, expected, span, help_text)
    }

    pub fn check_type_is_not_unknown(
        &self,
        id: TypeId,
        error_span: &Span,
    ) -> Result<TypeInfo, TypeError> {
        match self.look_up_type_id(id) {
            TypeInfo::Unknown => Err(TypeError::UnknownType {
                span: error_span.clone(),
            }),
            ty => Ok(ty),
        }
    }

    pub fn clear(&self) {
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
        let module_path = match type_info_prefix {
            Some(type_info_prefix) => type_info_prefix,
            None => mod_path,
        };
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
                    Some(TypedDeclaration::StructDeclaration(original_id)) => {
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
                    Some(TypedDeclaration::EnumDeclaration(original_id)) => {
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
                    Some(TypedDeclaration::GenericTypeForFunctionScope { name, type_id }) => {
                        self.insert_type(TypeInfo::Ref(type_id, name.span()))
                    }
                    _ => {
                        errors.push(CompileError::UnknownTypeName {
                            name: name.to_string(),
                            span: name.span(),
                        });
                        self.insert_type(TypeInfo::ErrorRecovery)
                    }
                }
            }
            TypeInfo::Ref(id, _) => id,
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
            o => insert_type(o),
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

pub fn look_up_type_id(id: TypeId) -> TypeInfo {
    TYPE_ENGINE.look_up_type_id(id)
}

pub(crate) fn look_up_type_id_raw(id: TypeId) -> TypeInfo {
    TYPE_ENGINE.look_up_type_id_raw(id)
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
    a: TypeId,
    b: TypeId,
    self_type: TypeId,
    span: &Span,
    help_text: impl Into<String>,
) -> (Vec<CompileWarning>, Vec<TypeError>) {
    TYPE_ENGINE.unify_with_self(a, b, self_type, span, help_text)
}

pub(crate) fn unify(
    a: TypeId,
    b: TypeId,
    span: &Span,
    help_text: impl Into<String>,
) -> (Vec<CompileWarning>, Vec<TypeError>) {
    TYPE_ENGINE.unify(a, b, span, help_text)
}

pub fn check_type_is_not_unknown(id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
    TYPE_ENGINE.check_type_is_not_unknown(id, error_span)
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

fn numeric_cast_compat(new_size: IntegerBits, old_size: IntegerBits) -> NumericCastCompatResult {
    // If this is a downcast, warn for loss of precision. If upcast, then no warning.
    use IntegerBits::*;
    match (new_size, old_size) {
        // These should generate a downcast warning.
        (Eight, Sixteen)
        | (Eight, ThirtyTwo)
        | (Eight, SixtyFour)
        | (Sixteen, ThirtyTwo)
        | (Sixteen, SixtyFour)
        | (ThirtyTwo, SixtyFour) => {
            NumericCastCompatResult::CastableWithWarning(Warning::LossOfPrecision {
                initial_type: old_size,
                cast_to: new_size,
            })
        }
        // Upcasting is ok, so everything else is ok.
        _ => NumericCastCompatResult::Compatible,
    }
}
enum NumericCastCompatResult {
    Compatible,
    CastableWithWarning(Warning),
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
