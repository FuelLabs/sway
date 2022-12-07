use core::fmt::Write;
use core::hash::Hasher;
use hashbrown::hash_map::RawEntryMut;
use hashbrown::HashMap;
use std::hash::BuildHasher;
use std::sync::RwLock;

use crate::concurrent_slab::ListDisplay;
use crate::{
    concurrent_slab::ConcurrentSlab, declaration_engine::*, engine_threading::*, language::ty,
    namespace::Path, type_system::*, Namespace,
};

use sway_error::{error::CompileError, type_error::TypeError, warning::CompileWarning};
use sway_types::{span::Span, Ident, Spanned};

#[derive(Debug, Default)]
pub struct TypeEngine {
    pub(super) slab: ConcurrentSlab<TypeInfo>,
    storage_only_types: ConcurrentSlab<TypeInfo>,
    id_map: RwLock<HashMap<TypeInfo, TypeId>>,
    unify_map: RwLock<HashMap<TypeId, Vec<TypeId>>>,
}

fn make_hasher<'a: 'b, 'b, K>(
    hash_builder: &'a impl BuildHasher,
    type_engine: &'b TypeEngine,
) -> impl Fn(&K) -> u64 + 'b
where
    K: HashWithEngines + ?Sized,
{
    move |key: &K| {
        let mut state = hash_builder.build_hasher();
        key.hash(&mut state, type_engine);
        state.finish()
    }
}

impl TypeEngine {
    /// Inserts a [TypeInfo] into the [TypeEngine] and returns a [TypeId]
    /// referring to that [TypeInfo].
    pub(crate) fn insert_type(&self, ty: TypeInfo) -> TypeId {
        let mut id_map = self.id_map.write().unwrap();

        let hash_builder = id_map.hasher().clone();
        let ty_hash = make_hasher(&hash_builder, self)(&ty);

        let raw_entry = id_map
            .raw_entry_mut()
            .from_hash(ty_hash, |x| x.eq(&ty, Engines::new(self, todo!())));
        match raw_entry {
            RawEntryMut::Occupied(o) => return *o.get(),
            RawEntryMut::Vacant(_) if ty.can_change() => TypeId::new(self.slab.insert(ty)),
            RawEntryMut::Vacant(v) => {
                let type_id = TypeId::new(self.slab.insert(ty.clone()));
                v.insert_with_hasher(ty_hash, ty, type_id, make_hasher(&hash_builder, self));
                type_id
            }
        }
    }

    pub(crate) fn insert_unified_type(&self, received: TypeId, expected: TypeId) {
        let mut unify_map = self.unify_map.write().unwrap();
        if let Some(type_ids) = unify_map.get(&received) {
            if type_ids.contains(&expected) {
                return;
            }
            let mut type_ids = type_ids.clone();
            type_ids.push(expected);
            unify_map.insert(received, type_ids);
            return;
        }

        unify_map.insert(received, vec![expected]);
    }

    pub(crate) fn get_unified_types(&self, type_id: TypeId) -> Vec<TypeId> {
        let mut final_unify_ids: Vec<TypeId> = vec![];
        self.get_unified_types_rec(type_id, &mut final_unify_ids);
        final_unify_ids
    }

    fn get_unified_types_rec(&self, type_id: TypeId, final_unify_ids: &mut Vec<TypeId>) {
        let unify_map = self.unify_map.read().unwrap();
        if let Some(unify_ids) = unify_map.get(&type_id) {
            for unify_id in unify_ids {
                if final_unify_ids.contains(unify_id) {
                    continue;
                }
                final_unify_ids.push(*unify_id);
                self.get_unified_types_rec(*unify_id, final_unify_ids);
            }
        }
    }

    /// Currently the [TypeEngine] is a lazy static object, so when we run
    /// cargo tests, we can either choose to use a local [TypeEngine] and bypass
    /// all of the global methods or we can use the lazy static [TypeEngine].
    /// This method is for testing to be able to bypass the global methods for
    /// the lazy static [TypeEngine] (contained within the call to hash in the
    /// id_map).
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn insert_type_always(&self, ty: TypeInfo) -> TypeId {
        TypeId::new(self.slab.insert(ty))
    }

    /// Performs a lookup of `id` into the [TypeEngine].
    pub fn look_up_type_id(&self, id: TypeId) -> TypeInfo {
        self.slab.get(id.index())
    }

    /// Denotes the given [TypeId] as being used with storage.
    pub(crate) fn set_type_as_storage_only(&self, id: TypeId) {
        self.storage_only_types.insert(self.look_up_type_id(id));
    }

    /// Checks if the given [TypeInfo] is a storage only type.
    pub(crate) fn is_type_info_storage_only(&self, ti: &TypeInfo) -> bool {
        self.storage_only_types
            .exists(|x| ti.is_subset_of(x, Engines::new(self, todo!())))
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn monomorphize<T>(
        &self,
        declaration_engine: &DeclarationEngine,
        value: &mut T,
        type_arguments: &mut [TypeArgument],
        enforce_type_arguments: EnforceTypeArguments,
        call_site_span: &Span,
        namespace: &mut Namespace,
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
                let type_mapping = TypeMapping::from_type_parameters(value.type_parameters(), self);
                value.copy_types(&type_mapping, Engines::new(self, declaration_engine));
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
                            declaration_engine,
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
                let type_mapping = TypeMapping::from_type_parameters_and_type_arguments(
                    value
                        .type_parameters()
                        .iter()
                        .map(|type_param| type_param.type_id)
                        .collect(),
                    type_arguments
                        .iter()
                        .map(|type_arg| type_arg.type_id)
                        .collect(),
                );
                value.copy_types(&type_mapping, Engines::new(self, declaration_engine));
                ok((), warnings, errors)
            }
        }
    }

    /// Replace any instances of the [TypeInfo::SelfType] variant with
    /// `self_type` in both `received` and `expected`, then unify `received` and
    /// `expected`.
    pub(crate) fn unify_with_self(
        &self,
        declaration_engine: &DeclarationEngine,
        mut received: TypeId,
        mut expected: TypeId,
        self_type: TypeId,
        span: &Span,
        help_text: &str,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        received.replace_self_type(Engines::new(self, declaration_engine), self_type);
        expected.replace_self_type(Engines::new(self, declaration_engine), self_type);
        self.unify(declaration_engine, received, expected, span, help_text)
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
        declaration_engine: &DeclarationEngine,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        normalize_err(unify::unify(
            Engines::new(self, declaration_engine),
            received,
            expected,
            span,
            help_text,
            false,
        ))
    }

    /// Replace any instances of the [TypeInfo::SelfType] variant with
    /// `self_type` in both `received` and `expected`, then unify_right
    /// `received` and `expected`.
    pub(crate) fn unify_right_with_self(
        &self,
        declaration_engine: &DeclarationEngine,
        mut received: TypeId,
        mut expected: TypeId,
        self_type: TypeId,
        span: &Span,
        help_text: &str,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        received.replace_self_type(Engines::new(self, declaration_engine), self_type);
        expected.replace_self_type(Engines::new(self, declaration_engine), self_type);
        self.unify_right(declaration_engine, received, expected, span, help_text)
    }

    /// Make the type of `expected` equivalent to `received`.
    ///
    /// This is different than the `unify` method because it _only allows
    /// changes to `expected`_. It also rejects the case where `received` is a
    /// generic type and `expected` is not a generic type.
    ///
    /// Here is an example for why this method is necessary. Take this Sway
    /// code:
    ///
    /// ```ignore
    /// fn test_function<T>(input: T) -> T {
    ///     input
    /// }
    ///
    /// fn call_it() -> bool {
    ///     test_function(true)
    /// }
    /// ```
    ///
    /// This is valid Sway code and we should expect it to compile because the
    /// type `bool` is valid under the generic type `T`.
    ///
    /// Now, look at this Sway code:
    ///
    /// ```ignore
    /// fn test_function(input: bool) -> bool {
    ///     input
    /// }
    ///
    /// fn call_it<T>(input: T) -> T {
    ///     test_function(input)
    /// }
    /// ```
    ///
    /// We should expect this Sway to fail to compile because the generic type
    /// `T` is not valid under the type `bool`.
    ///
    /// This is the function that makes that distinction for us!
    pub(crate) fn unify_right(
        &self,
        declaration_engine: &DeclarationEngine,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        normalize_err(unify::unify_right(
            Engines::new(self, declaration_engine),
            received,
            expected,
            span,
            help_text,
        ))
    }

    /// Helper function for making the type of `expected` equivalent to
    /// `received` for instantiating algebraic data types.
    ///
    /// This method simply switches the arguments of `received` and `expected`
    /// and calls the `unify` method---the main purpose of this method is reduce
    /// developer overhead during implementation, as it is a little non-intuitive
    /// why `received` and `expected` should be switched.
    ///
    /// Let me explain, take this Sway code:
    ///
    /// ```ignore
    /// enum Option<T> {
    ///     Some(T),
    ///     None
    /// }
    ///
    /// struct Wrapper {
    ///     option: Option<bool>,
    /// }
    ///
    /// fn create_it<T>() -> Wrapper {
    ///     Wrapper {
    ///         option: Option::None
    ///     }
    /// }
    /// ```
    ///
    /// This is valid Sway code and we should expect it to compile. Here is the
    /// pseudo-code of roughly what we can expect from type inference:
    /// 1. `Option::None` is originally found to be of type `Option<T>` (because
    ///     it is not possible to know what `T` is just from the `None` case)
    /// 2. we call `unify_adt` with arguments `received` of type `Option<T>` and
    ///     `expected` of type `Option<bool>`
    /// 3. we switch `received` and `expected` and call the `unify` method
    /// 4. we perform type inference with a `received` type of `Option<bool>`
    ///     and an `expected` type of `Option<T>`
    /// 5. we perform type inference with a `received` type of `bool` and an
    ///     `expected` type of `T`
    /// 6. because we have called the `unify` method (and not the `unify_right`
    ///     method), we can replace `T` with `bool`
    ///
    /// What's important about this is flipping the arguments prioritizes
    /// unifying `expected`, meaning if both `received` and `expected` are
    /// generic types, then `expected` will be replaced with `received`.
    pub(crate) fn unify_adt(
        &self,
        declaration_engine: &DeclarationEngine,
        received: TypeId,
        expected: TypeId,
        span: &Span,
        help_text: &str,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        normalize_err(unify::unify(
            Engines::new(self, declaration_engine),
            expected,
            received,
            span,
            help_text,
            true,
        ))
    }

    pub(crate) fn to_typeinfo(&self, id: TypeId, error_span: &Span) -> Result<TypeInfo, TypeError> {
        match self.look_up_type_id(id) {
            TypeInfo::Unknown => Err(TypeError::UnknownType {
                span: error_span.clone(),
            }),
            ty => Ok(ty),
        }
    }

    /// Resolve the type of the given [TypeId], replacing any instances of
    /// [TypeInfo::Custom] with either a monomorphized struct, monomorphized
    /// enum, or a reference to a type parameter.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_type(
        &self,
        declaration_engine: &DeclarationEngine,
        type_id: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
        namespace: &mut Namespace,
        mod_path: &Path,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let module_path = type_info_prefix.unwrap_or(mod_path);
        let type_id = match self.look_up_type_id(type_id) {
            TypeInfo::Custom {
                name,
                type_arguments,
            } => {
                match namespace
                    .root()
                    .resolve_symbol(module_path, &name)
                    .ok(&mut warnings, &mut errors)
                    .cloned()
                {
                    Some(ty::TyDeclaration::StructDeclaration(original_id)) => {
                        // get the copy from the declaration engine
                        let mut new_copy = check!(
                            CompileResult::from(
                                declaration_engine.get_struct(original_id, &name.span())
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );

                        // monomorphize the copy, in place
                        check!(
                            self.monomorphize(
                                declaration_engine,
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
                        let type_id = new_copy.create_type_id(self);

                        // take any trait methods that apply to this type and copy them to the new type
                        namespace.insert_trait_implementation_for_type(
                            Engines::new(self, declaration_engine),
                            type_id,
                        );

                        // return the id
                        type_id
                    }
                    Some(ty::TyDeclaration::EnumDeclaration(original_id)) => {
                        // get the copy from the declaration engine
                        let mut new_copy = check!(
                            CompileResult::from(
                                declaration_engine.get_enum(original_id, &name.span())
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );

                        // monomorphize the copy, in place
                        check!(
                            self.monomorphize(
                                declaration_engine,
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
                        let type_id = new_copy.create_type_id(self);

                        // take any trait methods that apply to this type and copy them to the new type
                        namespace.insert_trait_implementation_for_type(
                            Engines::new(self, declaration_engine),
                            type_id,
                        );

                        // return the id
                        type_id
                    }
                    Some(ty::TyDeclaration::GenericTypeForFunctionScope { type_id, .. }) => type_id,
                    _ => {
                        errors.push(CompileError::UnknownTypeName {
                            name: name.to_string(),
                            span: name.span(),
                        });
                        self.insert_type(TypeInfo::ErrorRecovery)
                    }
                }
            }
            TypeInfo::Array(mut elem_ty, n) => {
                elem_ty.type_id = check!(
                    self.resolve_type(
                        declaration_engine,
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
                self.insert_type(TypeInfo::Array(elem_ty, n))
            }
            TypeInfo::Tuple(mut type_arguments) => {
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = check!(
                        self.resolve_type(
                            declaration_engine,
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
    pub(crate) fn resolve_type_with_self(
        &self,
        declaration_engine: &DeclarationEngine,
        mut type_id: TypeId,
        self_type: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
        namespace: &mut Namespace,
        mod_path: &Path,
    ) -> CompileResult<TypeId> {
        type_id.replace_self_type(Engines::new(self, declaration_engine), self_type);
        self.resolve_type(
            declaration_engine,
            type_id,
            span,
            enforce_type_arguments,
            type_info_prefix,
            namespace,
            mod_path,
        )
    }

    /// Pretty print method for printing the [TypeEngine]. This method is
    /// manually implemented to avoid implementation overhead regarding using
    /// [DisplayWithEngines].
    pub fn pretty_print(&self, declaration_engine: &DeclarationEngine) -> String {
        let engines = Engines::new(self, declaration_engine);
        let mut builder = String::new();
        self.slab.with_slice(|elems| {
            let list = elems.iter().map(|type_info| engines.help_out(type_info));
            let list = ListDisplay { list };
            write!(builder, "TypeEngine {{\n{}\n}}", list).unwrap();
        });
        builder
    }
}

fn normalize_err(
    (w, e): (Vec<CompileWarning>, Vec<TypeError>),
) -> (Vec<CompileWarning>, Vec<CompileError>) {
    (w, e.into_iter().map(CompileError::from).collect())
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
