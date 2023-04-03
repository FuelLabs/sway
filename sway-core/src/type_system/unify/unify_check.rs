use crate::{decl_engine::*, engine_threading::*, type_system::priv_prelude::*};

/// Helper struct to aid in type coercion.
pub(crate) struct UnifyCheck<'a> {
    engines: Engines<'a>,
}

impl<'a> UnifyCheck<'a> {
    /// Creates a new [UnifyCheck].
    pub(crate) fn new(engines: Engines<'a>) -> UnifyCheck<'a> {
        UnifyCheck { engines }
    }

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
    pub(crate) fn check(&self, left: TypeId, right: TypeId) -> bool {
        use TypeInfo::*;

        if left == right {
            return true;
        }

        let left_info = self.engines.te().get(left);
        let right_info = self.engines.te().get(right);
        match (left_info, right_info) {
            (TypeParam { .. }, _) => {
                panic!();
                // false
            }
            (_, TypeParam { .. }) => {
                panic!();
                // false
            }

            // the placeholder type can be coerced into any type
            (Placeholder(_), _) => true,
            // any type can be coerced into the placeholder type
            (_, Placeholder(_)) => true,

            // Type aliases and the types they encapsulate coerce to each other.
            (Alias { ty, .. }, _) => self.check(ty.type_id, right),
            (_, Alias { ty, .. }) => self.check(left, ty.type_id),

            (
                UnknownGeneric {
                    name: ln,
                    trait_constraints: ltc,
                },
                UnknownGeneric {
                    name: rn,
                    trait_constraints: rtc,
                },
            ) => {
                // TODO: this requirement on the trait constraints should be
                // loosened to match the description above
                ln == rn && rtc.eq(&ltc, self.engines)
            }
            // any type can be coerced into generic
            (_, UnknownGeneric { .. }) => true,

            (Unknown, _) => true,
            (_, Unknown) => true,

            (Boolean, Boolean) => true,
            (SelfType, SelfType) => true,
            (B256, B256) => true,
            (Numeric, Numeric) => true,
            (Contract, Contract) => true,
            (RawUntypedPtr, RawUntypedPtr) => true,
            (RawUntypedSlice, RawUntypedSlice) => true,
            (UnsignedInteger(_), UnsignedInteger(_)) => true,
            (Numeric, UnsignedInteger(_)) => true,
            (UnsignedInteger(_), Numeric) => true,
            (Str(l), Str(r)) => l.val() == r.val(),

            (Array(l0, l1), Array(r0, r1)) => {
                self.check(l0.type_id, r0.type_id) && l1.val() == r1.val()
            }
            (Tuple(l_types), Tuple(r_types)) => {
                let l_types = l_types.iter().map(|x| x.type_id).collect::<Vec<_>>();
                let r_types = r_types.iter().map(|x| x.type_id).collect::<Vec<_>>();
                self.check_multiple(&l_types, &r_types)
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
                l_name.suffix == r_name.suffix && self.check_multiple(&l_types, &r_types)
            }
            // Let empty enums to coerce to any other type. This is useful for Never enum.
            (Enum(r_decl_ref), _)
                if self.engines.de().get_enum(&r_decl_ref).variants.is_empty() =>
            {
                true
            }
            (Enum(l), Enum(r)) => self.check_decl_ref(l, r),
            (Struct(l), Struct(r)) => self.check_decl_ref(l, r),

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

            // this is kinda a hack
            (ErrorRecovery, _) => true,
            (_, ErrorRecovery) => true,

            (a, b) => a.eq(&b, self.engines),
        }
    }

    fn check_decl_ref<T>(&self, left: DeclRef<DeclId<T>>, right: DeclRef<DeclId<T>>) -> bool {
        left.id() == right.id() && self.check_subst_list(left.subst_list(), right.subst_list())
    }

    fn check_subst_list(&self, left: &SubstList, right: &SubstList) -> bool {
        let preprocess = |subst_list: &SubstList| -> Vec<TypeId> {
            subst_list
                .elems()
                .into_iter()
                .map(|type_param| type_param.type_id)
                .collect()
        };

        left.len() == right.len() && self.check_multiple(&preprocess(left), &preprocess(right))
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
            if !self.check(*l, *r) {
                return false;
            }
        }

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
                if matches!(a, Placeholder(_)) || matches!(b, Placeholder(_)) {
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
            if matches!(a, Placeholder(_)) || matches!(b, Placeholder(_)) {
                continue;
            }
            if !a.eq(b, self.engines) {
                return false;
            }
        }

        // if all of the invariants are met, then `self` can be coerced into
        // `other`!
        true
    }
}
