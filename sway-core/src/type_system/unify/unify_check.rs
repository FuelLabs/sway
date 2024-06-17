use crate::{
    engine_threading::{Engines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::ty::{TyEnumDecl, TyStructDecl},
    type_system::{priv_prelude::*, unify::occurs_check::OccursCheck},
};

#[derive(Debug)]
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
        use UnifyCheckMode::NonGenericConstraintSubset;
        if left == right {
            return true;
        }

        let left_info = self.engines.te().get(left);
        let right_info = self.engines.te().get(right);

        // override top level generics with simple equality but only at top level
        if let NonGenericConstraintSubset = self.mode {
            if let UnknownGeneric { .. } = &*right_info {
                return left_info.eq(&right_info, &PartialEqWithEnginesContext::new(self.engines));
            }
        }
        self.check_inner(left, right)
    }

    fn check_inner(&self, left: TypeId, right: TypeId) -> bool {
        use TypeInfo::{
            Alias, Array, ContractCaller, Custom, Enum, ErrorRecovery, Never, Numeric, Placeholder,
            Ref, StringArray, StringSlice, Struct, Tuple, Unknown, UnknownGeneric, UnsignedInteger,
        };
        use UnifyCheckMode::{
            Coercion, ConstraintSubset, NonDynamicEquality, NonGenericConstraintSubset,
        };

        if left == right {
            return true;
        }

        let left_info = self.engines.te().get(left);
        let right_info = self.engines.te().get(right);

        // common recursion patterns
        match (&*left_info, &*right_info) {
            (Never, Never) => {
                return true;
            }
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

                return self.check_structs(&l_decl, &r_decl);
            }

            (
                Custom {
                    qualified_call_path: l_name,
                    type_arguments: l_type_args,
                    root_type_id: l_root_type_id,
                },
                Custom {
                    qualified_call_path: r_name,
                    type_arguments: r_type_args,
                    root_type_id: r_root_type_id,
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
                let l_root_type_ids = if let Some(l_root_type_id) = l_root_type_id {
                    vec![*l_root_type_id]
                } else {
                    vec![]
                };
                let r_root_type_ids = if let Some(r_root_type_id) = r_root_type_id {
                    vec![*r_root_type_id]
                } else {
                    vec![]
                };
                let same_qualified_path_root = match (
                    l_name.qualified_path_root.clone(),
                    r_name.qualified_path_root.clone(),
                ) {
                    (Some(l_qualified_path_root), Some(r_qualified_path_root)) => {
                        self.check_inner(
                            l_qualified_path_root.ty.type_id,
                            r_qualified_path_root.ty.type_id,
                        ) && self.check_inner(
                            l_qualified_path_root.as_trait,
                            r_qualified_path_root.as_trait,
                        )
                    }
                    (None, None) => true,
                    _ => false,
                };

                return l_name.call_path.suffix == r_name.call_path.suffix
                    && same_qualified_path_root
                    && self.check_multiple(&l_types, &r_types)
                    && self.check_multiple(&l_root_type_ids, &r_root_type_ids);
            }
            (Enum(l_decl_ref), Enum(r_decl_ref)) => {
                let l_decl = self.engines.de().get_enum(l_decl_ref);
                let r_decl = self.engines.de().get_enum(r_decl_ref);

                return self.check_enums(&l_decl, &r_decl);
            }

            (
                Ref {
                    to_mutable_value: l_to_mut,
                    referenced_type: l_ty,
                },
                Ref {
                    to_mutable_value: r_to_mut,
                    referenced_type: r_ty,
                },
            ) => {
                // Unification is possible in these situations, assuming that the referenced types
                // can unify:
                //     l  ->  r
                //  - `&` -> `&`
                //  - `&mut` -> `&`
                //  - `&mut` -> `&mut`
                return (*l_to_mut || !*r_to_mut) && self.check_inner(l_ty.type_id, r_ty.type_id);
            }

            (UnknownGeneric { parent: lp, .. }, r)
                if lp.is_some()
                    && self
                        .engines
                        .te()
                        .get(lp.unwrap())
                        .eq(r, &PartialEqWithEnginesContext::new(self.engines)) =>
            {
                return true;
            }
            (l, UnknownGeneric { parent: rp, .. })
                if rp.is_some()
                    && self
                        .engines
                        .te()
                        .get(rp.unwrap())
                        .eq(l, &PartialEqWithEnginesContext::new(self.engines)) =>
            {
                return true;
            }
            (UnknownGeneric { parent: lp, .. }, UnknownGeneric { parent: rp, .. })
                if lp.is_some()
                    && rp.is_some()
                    && self.engines.te().get(lp.unwrap()).eq(
                        &*self.engines.te().get(rp.unwrap()),
                        &PartialEqWithEnginesContext::new(self.engines),
                    ) =>
            {
                return true;
            }

            _ => {}
        }

        match self.mode {
            Coercion => {
                match (&*left_info, &*right_info) {
                    (r @ UnknownGeneric { .. }, e @ UnknownGeneric { .. })
                        if TypeInfo::is_self_type(r) || TypeInfo::is_self_type(e) =>
                    {
                        true
                    }
                    (
                        UnknownGeneric {
                            name: ln,
                            trait_constraints: ltc,
                            parent: _,
                            is_from_type_parameter: _,
                        },
                        UnknownGeneric {
                            name: rn,
                            trait_constraints: rtc,
                            parent: _,
                            is_from_type_parameter: _,
                        },
                    ) => ln == rn && rtc.eq(ltc, &PartialEqWithEnginesContext::new(self.engines)),
                    // any type can be coerced into a generic,
                    // except if the type already contains the generic
                    (_e, _g @ UnknownGeneric { .. }) => {
                        !OccursCheck::new(self.engines).check(right, left)
                    }

                    // Never coerces to any other type.
                    (Never, _) => true,

                    // the placeholder type can be coerced into any type
                    (Placeholder(_), _) => true,
                    // any type can be coerced into the placeholder type
                    (_, Placeholder(_)) => true,

                    // Type aliases and the types they encapsulate coerce to each other.
                    (Alias { ty, .. }, _) => self.check_inner(ty.type_id, right),
                    (_, Alias { ty, .. }) => self.check_inner(left, ty.type_id),

                    (Unknown, _) => true,
                    (_, Unknown) => true,

                    (UnsignedInteger(lb), UnsignedInteger(rb)) => lb == rb,
                    (Numeric, UnsignedInteger(_)) => true,
                    (UnsignedInteger(_), Numeric) => true,

                    (StringSlice, StringSlice) => true,
                    (StringArray(l), StringArray(r)) => l.val() == r.val(),

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
                        r.eq(e, &PartialEqWithEnginesContext::new(self.engines))
                            || (ran == ean && ra.is_none())
                            || matches!(ran, AbiName::Deferred)
                            || (ran == ean && ea.is_none())
                            || matches!(ean, AbiName::Deferred)
                    }

                    (ErrorRecovery(_), _) => true,
                    (_, ErrorRecovery(_)) => true,

                    (a, b) => a.eq(b, &PartialEqWithEnginesContext::new(self.engines)),
                }
            }
            ConstraintSubset | NonGenericConstraintSubset => {
                match (&*left_info, &*right_info) {
                    (
                        UnknownGeneric {
                            name: _,
                            trait_constraints: ltc,
                            parent: _,
                            is_from_type_parameter: _,
                        },
                        UnknownGeneric {
                            name: _,
                            trait_constraints: rtc,
                            parent: _,
                            is_from_type_parameter: _,
                        },
                    ) => rtc.eq(ltc, &PartialEqWithEnginesContext::new(self.engines)),

                    // any type can be coerced into a generic,
                    // except if the type already contains the generic
                    (_e, _g @ UnknownGeneric { .. }) => {
                        !OccursCheck::new(self.engines).check(right, left)
                    }

                    (Alias { ty: l_ty, .. }, Alias { ty: r_ty, .. }) => {
                        self.check_inner(l_ty.type_id, r_ty.type_id)
                    }
                    (a, b) => a.eq(b, &PartialEqWithEnginesContext::new(self.engines)),
                }
            }
            NonDynamicEquality => match (&*left_info, &*right_info) {
                // when a type alias is encountered, defer the decision to the type it contains (i.e. the
                // type it aliases with)
                (Alias { ty, .. }, _) => self.check_inner(ty.type_id, right),
                (_, Alias { ty, .. }) => self.check_inner(left, ty.type_id),

                // these cases are false because, unless left and right have the same
                // TypeId, they may later resolve to be different types in the type
                // engine
                (TypeInfo::Unknown, TypeInfo::Unknown) => false,
                (TypeInfo::Numeric, TypeInfo::Numeric) => false,
                (TypeInfo::Storage { .. }, TypeInfo::Storage { .. }) => false,

                // these cases are able to be directly compared
                (TypeInfo::Contract, TypeInfo::Contract) => true,
                (TypeInfo::Boolean, TypeInfo::Boolean) => true,
                (TypeInfo::B256, TypeInfo::B256) => true,
                (TypeInfo::ErrorRecovery(_), TypeInfo::ErrorRecovery(_)) => true,
                (TypeInfo::StringSlice, TypeInfo::StringSlice) => true,
                (TypeInfo::StringArray(l), TypeInfo::StringArray(r)) => l.val() == r.val(),
                (TypeInfo::UnsignedInteger(l), TypeInfo::UnsignedInteger(r)) => l == r,
                (TypeInfo::RawUntypedPtr, TypeInfo::RawUntypedPtr) => true,
                (TypeInfo::RawUntypedSlice, TypeInfo::RawUntypedSlice) => true,
                (
                    TypeInfo::UnknownGeneric {
                        name: rn,
                        trait_constraints: rtc,
                        parent: _,
                        is_from_type_parameter: _,
                    },
                    TypeInfo::UnknownGeneric {
                        name: en,
                        trait_constraints: etc,
                        parent: _,
                        is_from_type_parameter: _,
                    },
                ) => {
                    rn.as_str() == en.as_str()
                        && rtc.eq(etc, &PartialEqWithEnginesContext::new(self.engines))
                }
                (TypeInfo::Placeholder(_), TypeInfo::Placeholder(_)) => false,
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
                        && Option::zip(l_address.clone(), r_address.clone())
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
    /// 1. `left` and `right` are of the same length _n_
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
        use TypeInfo::{Numeric, Placeholder, Unknown, UnsignedInteger};
        use UnifyCheckMode::{
            Coercion, ConstraintSubset, NonDynamicEquality, NonGenericConstraintSubset,
        };

        // invariant 1. `left` and `right` are of the same length _n_
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
                            && (matches!(
                                (&**a, &**b),
                                (_, Placeholder(_))
                                    | (Placeholder(_), _)
                                    | (UnsignedInteger(_), Numeric)
                                    | (Numeric, UnsignedInteger(_))
                                    | (_, Unknown)
                                    | (Unknown, _)
                            ))
                        {
                            continue;
                        }
                        if a.eq(b, &PartialEqWithEnginesContext::new(self.engines)) {
                            // if a and b are the same type
                            constraints.push((i, j));
                        }
                    }
                }
                for (i, j) in &constraints {
                    let a = left_types.get(*i).unwrap();
                    let b = left_types.get(*j).unwrap();
                    if matches!(&self.mode, Coercion)
                        && (matches!(
                            (&**a, &**b),
                            (_, Placeholder(_))
                                | (Placeholder(_), _)
                                | (UnsignedInteger(_), Numeric)
                                | (Numeric, UnsignedInteger(_))
                                | (_, Unknown)
                                | (Unknown, _)
                        ))
                    {
                        continue;
                    }
                    if !a.eq(b, &PartialEqWithEnginesContext::new(self.engines)) {
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

    pub(crate) fn check_enums(&self, left: &TyEnumDecl, right: &TyEnumDecl) -> bool {
        assert!(
            left.call_path.is_absolute && right.call_path.is_absolute,
            "The call paths of the enum declarations must always be absolute."
        );

        // Avoid unnecessary `collect::<Vec>>` of variant names
        // and enum type parameters by short-circuiting.

        if left.call_path != right.call_path {
            return false;
        }

        // TODO: Is checking of variants necessary? Can we have two enums with the same `call_path`
        //       with different variants?
        //       We can have multiple declarations in a file and "already exist" errors, but those
        //       different declarations shouldn't reach the type checking phase. The last one
        //       will always win.
        if left.variants.len() != right.variants.len() {
            return false;
        }

        // Cheap name check first.
        if left
            .variants
            .iter()
            .zip(right.variants.iter())
            .any(|(l, r)| l.name != r.name)
        {
            return false;
        }

        if left
            .variants
            .iter()
            .zip(right.variants.iter())
            .any(|(l, r)| !self.check_inner(l.type_argument.type_id, r.type_argument.type_id))
        {
            return false;
        }

        if left.type_parameters.len() != right.type_parameters.len() {
            return false;
        }

        let l_types = left
            .type_parameters
            .iter()
            .map(|x| x.type_id)
            .collect::<Vec<_>>();

        let r_types = right
            .type_parameters
            .iter()
            .map(|x| x.type_id)
            .collect::<Vec<_>>();

        self.check_multiple(&l_types, &r_types)
    }

    pub(crate) fn check_structs(&self, left: &TyStructDecl, right: &TyStructDecl) -> bool {
        assert!(
            left.call_path.is_absolute && right.call_path.is_absolute,
            "The call paths of the struct declarations must always be absolute."
        );

        // Avoid unnecessary `collect::<Vec>>` of variant names
        // and enum type parameters by short-circuiting.

        if left.call_path != right.call_path {
            return false;
        }

        // TODO: Is checking of fields necessary? Can we have two structs with the same `call_path`
        //       with different fields?
        //       We can have multiple declarations in a file and "already exist" errors, but those
        //       different declarations shouldn't reach the type checking phase. The last one
        //       will always win.
        if left.fields.len() != right.fields.len() {
            return false;
        }

        // Cheap name check first.
        if left
            .fields
            .iter()
            .zip(right.fields.iter())
            .any(|(l, r)| l.name != r.name)
        {
            return false;
        }

        if left
            .fields
            .iter()
            .zip(right.fields.iter())
            .any(|(l, r)| !self.check_inner(l.type_argument.type_id, r.type_argument.type_id))
        {
            return false;
        }

        if left.type_parameters.len() != right.type_parameters.len() {
            return false;
        }

        let l_types = left
            .type_parameters
            .iter()
            .map(|x| x.type_id)
            .collect::<Vec<_>>();

        let r_types = right
            .type_parameters
            .iter()
            .map(|x| x.type_id)
            .collect::<Vec<_>>();

        self.check_multiple(&l_types, &r_types)
    }
}
