use crate::{engine_threading::*, type_system::priv_prelude::*};
use sway_types::Spanned;

enum UnifyCheckMode {
    /// Given two [TypeId]'s `left` and `right`, check to see if `left` can be
    /// coerced into `right`.
    ///
    /// `left` can be coerced into `right` if it can be generalized over
    /// `right`. For example, the generic `T` can be coerced into the
    /// placeholder type `_` or another generic with the same name and with
    /// certain compatible trait constraints. The type `u8` can also be coerced
    /// into the placeholder type `_` or a generic type. The placeholder type
    /// can be coerced into any type.
    ///
    /// Given:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///   x: T,
    ///   y: F,
    /// }
    /// ```
    ///
    /// the type `Data<T, F>` can be coerced into the placeholder type `_` or a
    /// generic type.
    ///
    /// Given:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///   x: T,
    ///   y: F,
    /// }
    ///
    /// impl<T> Data<T, T> { }
    /// ```
    ///
    /// the type `Data<T, T>` can be coerced into `Data<T, F>`, but
    /// _`Data<T, F>` cannot be coerced into `Data<T, T>`_.
    ///
    /// Given:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///   x: T,
    ///   y: F,
    /// }
    ///
    /// impl<T> Data<T, T> { }
    ///
    /// fn dummy() {
    ///     // the type of foo is Data<bool, u64>
    ///     let foo = Data {
    ///         x: true,
    ///         y: 1u64
    ///     };
    ///     // the type of bar is Data<u8, u8>
    ///     let bar = Data {
    ///         x: 0u8,
    ///         y: 0u8
    ///     };
    /// }
    /// ```
    ///
    /// then:
    ///
    /// | type:             | can be coerced into of:                                | can not be:         |
    /// |-------------------|--------------------------------------------------------|---------------------|
    /// | `Data<T, T>`      | `Data<T, F>`, any generic type, `_` type               |                     |
    /// | `Data<T, F>`      | any generic type, `_` type                             | `Data<T, T>`        |
    /// | `Data<bool, u64>` | `Data<T, F>`, any generic type, `_` type               | `Data<T, T>`        |
    /// | `Data<u8, u8>`    | `Data<T, T>`, `Data<T, F>`, any generic type, `_` type |                     |
    ///
    /// For generic types with trait constraints, the generic type `left` can be
    /// coerced into the generic type `right` when the trait constraints of
    /// `right` can be coerced into the trait constraints of `left`. This is a
    /// bit unintuitive, but you can think of it this way---a generic type
    /// `left` can be generalized over `right` when `right` has no methods
    /// that `left` doesn't have. These methods are coming from the trait
    /// constraints---if the trait constraints of `right` can be coerced into
    /// the trait constraints of `left`, then we know that `right` has unique
    /// methods.
    Coercion,
    /// Given two `TypeInfo`'s `self` and `other`, check to see if `self` is
    /// unidirectionally a subset of `other`.
    ///
    /// `self` is a subset of `other` if it can be generalized over `other`.
    /// For example, the generic `T` is a subset of the generic `F` because
    /// anything of the type `T` could also be of the type `F` (minus any
    /// external context that may make this statement untrue).
    ///
    /// Given:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///   x: T,
    ///   y: F,
    /// }
    /// ```
    ///
    /// the type `Data<T, F>` is a subset of any generic type.
    ///
    /// Given:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///   x: T,
    ///   y: F,
    /// }
    ///
    /// impl<T> Data<T, T> { }
    /// ```
    ///
    /// the type `Data<T, T>` is a subset of `Data<T, F>`, but _`Data<T, F>` is
    /// not a subset of `Data<T, T>`_.
    ///
    /// Given:
    ///
    /// ```ignore
    /// struct Data<T, F> {
    ///   x: T,
    ///   y: F,
    /// }
    ///
    /// impl<T> Data<T, T> { }
    ///
    /// fn dummy() {
    ///     // the type of foo is Data<bool, u64>
    ///     let foo = Data {
    ///         x: true,
    ///         y: 1u64
    ///     };
    ///     // the type of bar is Data<u8, u8>
    ///     let bar = Data {
    ///         x: 0u8,
    ///         y: 0u8
    ///     };
    /// }
    /// ```
    ///
    /// then:
    ///
    /// | type:             | is subset of:                                | is not a subset of: |
    /// |-------------------|----------------------------------------------|---------------------|
    /// | `Data<T, T>`      | `Data<T, F>`, any generic type               |                     |
    /// | `Data<T, F>`      | any generic type                             | `Data<T, T>`        |
    /// | `Data<bool, u64>` | `Data<T, F>`, any generic type               | `Data<T, T>`        |
    /// | `Data<u8, u8>`    | `Data<T, T>`, `Data<T, F>`, any generic type |                     |
    ///
    /// For generic types with trait constraints, the generic type `self` is a
    /// subset of the generic type `other` when the trait constraints of
    /// `other` are a subset of the trait constraints of `self`. This is a bit
    /// unintuitive, but you can think of it this way---a generic type `self`
    /// can be generalized over `other` when `other` has no methods
    /// that `self` doesn't have. These methods are coming from the trait
    /// constraints---if the trait constraints of `other` are a subset of the
    /// trait constraints of `self`, then we know that `other` has unique
    /// methods.
    ConstraintSubset,
    /// Given two `TypeInfo`'s `self` and `other`, checks to see if `self` is
    /// unidirectionally a subset of `other`, excluding consideration of generic
    /// types.
    NonGenericConstraintSubset,

    NonDynamicEquality,
}

/// Helper struct to aid in type coercion.
pub(crate) struct UnifyCheck<'a> {
    engines: &'a Engines,
    mode: UnifyCheckMode,
}

impl<'a> UnifyCheck<'a> {
    pub(crate) fn coercion(engines: &'a Engines) -> Self {
        Self {
            engines,
            mode: UnifyCheckMode::Coercion,
        }
    }
    pub(crate) fn constraint_subset(engines: &'a Engines) -> Self {
        Self {
            engines,
            mode: UnifyCheckMode::ConstraintSubset,
        }
    }
    pub(crate) fn non_generic_constraint_subset(engines: &'a Engines) -> Self {
        Self {
            engines,
            mode: UnifyCheckMode::NonGenericConstraintSubset,
        }
    }

    pub(crate) fn non_dynamic_equality(engines: &'a Engines) -> Self {
        Self {
            engines,
            mode: UnifyCheckMode::NonDynamicEquality,
        }
    }

    pub(crate) fn check(&self, left: TypeId, right: TypeId) -> bool {
        use TypeInfo::*;
        use UnifyCheckMode::*;
        if left == right {
            return true;
        }
        let left_info = self.engines.te().get(left);
        let right_info = self.engines.te().get(right);

        // override top level generics with simple equality but only at top level
        if let NonGenericConstraintSubset = self.mode {
            if let UnknownGeneric { .. } = right_info {
                return left_info.eq(&right_info, self.engines);
            }
        }
        self.check_inner(left, right)
    }

    fn check_inner(&self, left: TypeId, right: TypeId) -> bool {
        use TypeInfo::*;
        use UnifyCheckMode::*;

        if left == right {
            return true;
        }

        let left_info = self.engines.te().get(left);
        let right_info = self.engines.te().get(right);

        // common recursion patterns
        match (&left_info, &right_info) {
            (Array(l0, l1), Array(r0, r1)) => {
                return self.check_inner(l0.type_id, r0.type_id) && l1.val() == r1.val();
            }
            (Tuple(l_types), Tuple(r_types)) => {
                let l_types = l_types.iter().map(|x| x.type_id).collect::<Vec<_>>();
                let r_types = r_types.iter().map(|x| x.type_id).collect::<Vec<_>>();
                return self.check_multiple(&l_types, &r_types);
            }

            (Struct(l_decl_ref), Struct(r_decl_ref)) => {
                let l_decl = self.engines.de().get_struct(l_decl_ref);
                let r_decl = self.engines.de().get_struct(r_decl_ref);
                let l_names = l_decl
                    .fields
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let r_names = r_decl
                    .fields
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let l_types = l_decl
                    .type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let r_types = r_decl
                    .type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                return l_decl_ref.name().clone() == r_decl_ref.name().clone()
                    && l_names == r_names
                    && self.check_multiple(&l_types, &r_types);
            }
            (
                Custom {
                    call_path: l_name,
                    type_arguments: l_type_args,
                },
                Custom {
                    call_path: r_name,
                    type_arguments: r_type_args,
                },
            ) => {
                let l_types = l_type_args
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let r_types = r_type_args
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                return l_name.suffix == r_name.suffix && self.check_multiple(&l_types, &r_types);
            }
            _ => {}
        }

        match self.mode {
            Coercion => {
                match (left_info, right_info) {
                    (
                        UnknownGeneric {
                            name: ln,
                            trait_constraints: ltc,
                        },
                        UnknownGeneric {
                            name: rn,
                            trait_constraints: rtc,
                        },
                    ) => ln == rn && rtc.eq(&ltc, self.engines),
                    // any type can be coerced into generic
                    (_, UnknownGeneric { .. }) => true,

                    // Let empty enums to coerce to any other type. This is useful for Never enum.
                    (Enum(r_decl_ref), _)
                        if self.engines.de().get_enum(&r_decl_ref).variants.is_empty() =>
                    {
                        true
                    }
                    (Enum(l_decl_ref), Enum(r_decl_ref)) => {
                        let l_decl = self.engines.de().get_enum(&l_decl_ref);
                        let r_decl = self.engines.de().get_enum(&r_decl_ref);
                        let l_names = l_decl
                            .variants
                            .iter()
                            .map(|x| x.name.clone())
                            .collect::<Vec<_>>();
                        let r_names = r_decl
                            .variants
                            .iter()
                            .map(|x| x.name.clone())
                            .collect::<Vec<_>>();
                        let l_types = l_decl
                            .type_parameters
                            .iter()
                            .map(|x| x.type_id)
                            .collect::<Vec<_>>();
                        let r_types = r_decl
                            .type_parameters
                            .iter()
                            .map(|x| x.type_id)
                            .collect::<Vec<_>>();

                        l_decl_ref.name().clone() == r_decl_ref.name().clone()
                            && l_names == r_names
                            && self.check_multiple(&l_types, &r_types)
                    }

                    // the placeholder type can be coerced into any type
                    (Placeholder(_), _) => true,
                    // any type can be coerced into the placeholder type
                    (_, Placeholder(_)) => true,

                    // Type aliases and the types they encapsulate coerce to each other.
                    (Alias { ty, .. }, _) => self.check_inner(ty.type_id, right),
                    (_, Alias { ty, .. }) => self.check_inner(left, ty.type_id),

                    (Unknown, _) => true,
                    (_, Unknown) => true,

                    (UnsignedInteger(_), UnsignedInteger(_)) => true,
                    (Numeric, UnsignedInteger(_)) => true,
                    (UnsignedInteger(_), Numeric) => true,
                    (Str(l), Str(r)) => l.val() == r.val(),

                    // For contract callers, they can be coerced if they have the same
                    // name and at least one has an address of `None`
                    (
                        ref r @ ContractCaller {
                            abi_name: ref ran,
                            address: ref ra,
                        },
                        ref e @ ContractCaller {
                            abi_name: ref ean,
                            address: ref ea,
                        },
                    ) => {
                        r.eq(e, self.engines)
                            || (ran == ean && ra.is_none())
                            || matches!(ran, AbiName::Deferred)
                            || (ran == ean && ea.is_none())
                            || matches!(ean, AbiName::Deferred)
                    }

                    (ErrorRecovery, _) => true,
                    (_, ErrorRecovery) => true,

                    (a, b) => a.eq(&b, self.engines),
                }
            }
            ConstraintSubset | NonGenericConstraintSubset => {
                match (left_info, right_info) {
                    (
                        UnknownGeneric {
                            name: _,
                            trait_constraints: ltc,
                        },
                        UnknownGeneric {
                            name: _,
                            trait_constraints: rtc,
                        },
                    ) => rtc.eq(&ltc, self.engines),
                    // any type can be coerced into generic
                    (_, UnknownGeneric { .. }) => true,

                    (Enum(l_decl_ref), Enum(r_decl_ref)) => {
                        let l_decl = self.engines.de().get_enum(&l_decl_ref);
                        let r_decl = self.engines.de().get_enum(&r_decl_ref);
                        let l_names = l_decl
                            .variants
                            .iter()
                            .map(|x| x.name.clone())
                            .collect::<Vec<_>>();
                        let r_names = r_decl
                            .variants
                            .iter()
                            .map(|x| x.name.clone())
                            .collect::<Vec<_>>();
                        let l_types = l_decl
                            .type_parameters
                            .iter()
                            .map(|x| x.type_id)
                            .collect::<Vec<_>>();
                        let r_types = r_decl
                            .type_parameters
                            .iter()
                            .map(|x| x.type_id)
                            .collect::<Vec<_>>();

                        l_decl.call_path.suffix.span() == r_decl.call_path.suffix.span()
                            && l_decl_ref.name().clone() == r_decl_ref.name().clone()
                            && l_names == r_names
                            && self.check_multiple(&l_types, &r_types)
                    }

                    (Alias { ty: l_ty, .. }, Alias { ty: r_ty, .. }) => {
                        self.check_inner(l_ty.type_id, r_ty.type_id)
                    }
                    (a, b) => a.eq(&b, self.engines),
                }
            }
            NonDynamicEquality => match (left_info, right_info) {
                // when a type alias is encoutered, defer the decision to the type it contains (i.e. the
                // type it aliases with)
                (Alias { ty, .. }, _) => self.check_inner(ty.type_id, right),
                (_, Alias { ty, .. }) => self.check_inner(left, ty.type_id),

                // these cases are false because, unless left and right have the same
                // TypeId, they may later resolve to be different types in the type
                // engine
                (TypeInfo::Unknown, TypeInfo::Unknown) => false,
                (TypeInfo::SelfType, TypeInfo::SelfType) => false,
                (TypeInfo::Numeric, TypeInfo::Numeric) => false,
                (TypeInfo::Storage { .. }, TypeInfo::Storage { .. }) => false,

                // these cases are able to be directly compared
                (TypeInfo::Contract, TypeInfo::Contract) => true,
                (TypeInfo::Boolean, TypeInfo::Boolean) => true,
                (TypeInfo::B256, TypeInfo::B256) => true,
                (TypeInfo::ErrorRecovery, TypeInfo::ErrorRecovery) => true,
                (TypeInfo::Str(l), TypeInfo::Str(r)) => l.val() == r.val(),
                (TypeInfo::UnsignedInteger(l), TypeInfo::UnsignedInteger(r)) => l == r,
                (TypeInfo::RawUntypedPtr, TypeInfo::RawUntypedPtr) => true,
                (TypeInfo::RawUntypedSlice, TypeInfo::RawUntypedSlice) => true,
                (
                    TypeInfo::UnknownGeneric {
                        name: rn,
                        trait_constraints: rtc,
                    },
                    TypeInfo::UnknownGeneric {
                        name: en,
                        trait_constraints: etc,
                    },
                ) => rn.as_str() == en.as_str() && rtc.eq(&etc, self.engines),
                (TypeInfo::Placeholder(_), TypeInfo::Placeholder(_)) => false,

                (Enum(l_decl_ref), Enum(r_decl_ref)) => {
                    let l_decl = self.engines.de().get_enum(&l_decl_ref);
                    let r_decl = self.engines.de().get_enum(&r_decl_ref);
                    let l_names = l_decl
                        .variants
                        .iter()
                        .map(|x| x.name.clone())
                        .collect::<Vec<_>>();
                    let r_names = r_decl
                        .variants
                        .iter()
                        .map(|x| x.name.clone())
                        .collect::<Vec<_>>();
                    let l_types = l_decl
                        .type_parameters
                        .iter()
                        .map(|x| x.type_id)
                        .collect::<Vec<_>>();
                    let r_types = r_decl
                        .type_parameters
                        .iter()
                        .map(|x| x.type_id)
                        .collect::<Vec<_>>();

                    l_decl_ref.name().clone() == r_decl_ref.name().clone()
                        && l_names == r_names
                        && self.check_multiple(&l_types, &r_types)
                }

                (
                    TypeInfo::ContractCaller {
                        abi_name: l_abi_name,
                        address: l_address,
                    },
                    TypeInfo::ContractCaller {
                        abi_name: r_abi_name,
                        address: r_address,
                    },
                ) => {
                    l_abi_name == r_abi_name
                        && Option::zip(l_address, r_address)
                            .map(|(l_address, r_address)| {
                                self.check(l_address.return_type, r_address.return_type)
                            })
                            .unwrap_or(true)
                }

                _ => false,
            },
        }
    }

    /// Given two lists of [TypeId]'s `left` and `right`, check to see if
    /// `left` can be coerced into `right`.
    ///
    /// `left` can be coerced into `right` if the following invariants are true:
    /// 1. `left` and and `right` are of the same length _n_
    /// 2. For every _i_ in [0, n), `left`ᵢ can be coerced into `right`ᵢ
    /// 3. The elements of `left` satisfy the trait constraints of `right`
    ///
    /// A property that falls of out these constraints are that if `left` and
    /// `right` are empty, then `left` can be coerced into `right`.
    ///
    /// Given:
    ///
    /// ```ignore
    /// left:   [T]
    /// right:  [T, F]
    /// ```
    ///
    /// `left` cannot be coerced into `right` because it violates invariant #1.
    ///
    /// Given:
    ///
    /// ```ignore
    /// left:   [T, F]
    /// right:  [bool, F]
    /// ```
    ///
    /// `left` cannot be coerced into `right` because it violates invariant #2.
    ///
    /// Given:
    ///
    /// ```ignore
    /// left:   [T, F]
    /// right:  [T, T]
    /// ```
    ///
    /// `left` cannot be coerced into `right` because it violates invariant #3.
    ///
    /// Given:
    ///
    /// ```ignore
    /// left:   [T, T]
    /// right:  [T, F]
    /// ```
    ///
    /// `left` can be coerced into `right`.
    ///
    /// Given:
    ///
    /// ```ignore
    /// left:   [bool, T]
    /// right:  [T, F]
    /// ```
    ///
    /// `left` can be coerced into `right`.
    ///
    /// Given:
    ///
    /// ```ignore
    /// left:   [Data<T, T>, Data<T, F>]
    /// right:  [Data<T, F>, Data<T, F>]
    /// ```
    ///
    /// `left` can be coerced into `right`.
    ///
    fn check_multiple(&self, left: &[TypeId], right: &[TypeId]) -> bool {
        use TypeInfo::*;
        use UnifyCheckMode::*;

        // invariant 1. `left` and and `right` are of the same length _n_
        if left.len() != right.len() {
            return false;
        }

        // if `left` and `right` are empty, `left` can be coerced into `right`
        if left.is_empty() && right.is_empty() {
            return true;
        }

        // invariant 2. For every _i_ in [0, n), `left`ᵢ can be coerced into
        // `right`ᵢ
        for (l, r) in left.iter().zip(right.iter()) {
            if !self.check_inner(*l, *r) {
                return false;
            }
        }

        match self.mode {
            Coercion | ConstraintSubset | NonGenericConstraintSubset => {
                // invariant 3. The elements of `left` satisfy the constraints of `right`
                let left_types = left
                    .iter()
                    .map(|x| self.engines.te().get(*x))
                    .collect::<Vec<_>>();
                let right_types = right
                    .iter()
                    .map(|x| self.engines.te().get(*x))
                    .collect::<Vec<_>>();
                let mut constraints = vec![];
                for i in 0..(right_types.len() - 1) {
                    for j in (i + 1)..right_types.len() {
                        let a = right_types.get(i).unwrap();
                        let b = right_types.get(j).unwrap();
                        if matches!(&self.mode, Coercion)
                            && (matches!(a, Placeholder(_)) || matches!(b, Placeholder(_)))
                        {
                            continue;
                        }
                        if a.eq(b, self.engines) {
                            // if a and b are the same type
                            constraints.push((i, j));
                        }
                    }
                }
                for (i, j) in constraints.into_iter() {
                    let a = left_types.get(i).unwrap();
                    let b = left_types.get(j).unwrap();
                    if matches!(&self.mode, Coercion)
                        && (matches!(a, Placeholder(_)) || matches!(b, Placeholder(_)))
                    {
                        continue;
                    }
                    if !a.eq(b, self.engines) {
                        return false;
                    }
                }
            }
            // no constraint check, just propagate the check
            NonDynamicEquality => {}
        }

        // if all of the invariants are met, then `self` can be coerced into
        // `other`!
        true
    }
}
