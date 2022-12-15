use crate::type_system::*;

/// Helper struct to aid in type coercion.
pub(super) struct Coercion<'a> {
    type_engine: &'a TypeEngine,
}

impl<'a> Coercion<'a> {
    /// Creates a new [Coercion].
    pub(super) fn new(type_engine: &'a TypeEngine) -> Coercion<'a> {
        Coercion { type_engine }
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
    pub(super) fn check(&self, left: TypeId, right: TypeId) -> bool {
        let left = self.type_engine.look_up_type_id(left);
        let right = self.type_engine.look_up_type_id(right);
        match (left, right) {
            // the placeholder type can be coerced into any type
            (TypeInfo::Placeholder(_), _) => true,
            // any type can be coerced into the placeholder type
            (_, TypeInfo::Placeholder(_)) => true,

            (
                TypeInfo::UnknownGeneric {
                    name: ln,
                    trait_constraints: ltc,
                },
                TypeInfo::UnknownGeneric {
                    name: rn,
                    trait_constraints: rtc,
                },
            ) => {
                // TODO: this requirement on the trait constraints should be
                // loosened to match the description above
                ln == rn && rtc.eq(&ltc, self.type_engine)
            }
            // any type can be coerced into generic
            (_, TypeInfo::UnknownGeneric { .. }) => true,

            (TypeInfo::Unknown, _) => true,
            (_, TypeInfo::Unknown) => true,

            (TypeInfo::Boolean, TypeInfo::Boolean) => true,
            (TypeInfo::SelfType, TypeInfo::SelfType) => true,
            (TypeInfo::B256, TypeInfo::B256) => true,
            (TypeInfo::Numeric, TypeInfo::Numeric) => true,
            (TypeInfo::Contract, TypeInfo::Contract) => true,
            (TypeInfo::RawUntypedPtr, TypeInfo::RawUntypedPtr) => true,
            (TypeInfo::RawUntypedSlice, TypeInfo::RawUntypedSlice) => true,
            (TypeInfo::UnsignedInteger(_), TypeInfo::UnsignedInteger(_)) => true,
            (TypeInfo::Numeric, TypeInfo::UnsignedInteger(_)) => true,
            (TypeInfo::UnsignedInteger(_), TypeInfo::Numeric) => true,
            (TypeInfo::Str(l), TypeInfo::Str(r)) => l.val() == r.val(),

            (TypeInfo::Array(l0, l1), TypeInfo::Array(r0, r1)) => {
                self.check(l0.type_id, r0.type_id) && l1.val() == r1.val()
            }
            (TypeInfo::Tuple(l_types), TypeInfo::Tuple(r_types)) => {
                let l_types = l_types.iter().map(|x| x.type_id).collect::<Vec<_>>();
                let r_types = r_types.iter().map(|x| x.type_id).collect::<Vec<_>>();
                self.check_multiple(&l_types, &r_types)
            }

            (
                TypeInfo::Custom {
                    name: l_name,
                    type_arguments: l_type_args,
                },
                TypeInfo::Custom {
                    name: r_name,
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
                l_name == r_name && self.check_multiple(&l_types, &r_types)
            }
            (
                TypeInfo::Enum {
                    name: l_name,
                    variant_types: l_variant_types,
                    type_parameters: l_type_parameters,
                },
                TypeInfo::Enum {
                    name: r_name,
                    variant_types: r_variant_types,
                    type_parameters: r_type_parameters,
                },
            ) => {
                let l_names = l_variant_types
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let r_names = r_variant_types
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>();
                let l_types = l_type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let r_types = r_type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                l_name == r_name && l_names == r_names && self.check_multiple(&l_types, &r_types)
            }
            (
                TypeInfo::Struct {
                    name: l_name,
                    fields: l_fields,
                    type_parameters: l_type_parameters,
                },
                TypeInfo::Struct {
                    name: r_name,
                    fields: r_fields,
                    type_parameters: r_type_parameters,
                },
            ) => {
                let l_names = l_fields.iter().map(|x| x.name.clone()).collect::<Vec<_>>();
                let r_names = r_fields.iter().map(|x| x.name.clone()).collect::<Vec<_>>();
                let l_types = l_type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                let r_types = r_type_parameters
                    .iter()
                    .map(|x| x.type_id)
                    .collect::<Vec<_>>();
                l_name == r_name && l_names == r_names && self.check_multiple(&l_types, &r_types)
            }

            // For contract callers, they can be coerced if they have the same
            // name and at least one has an address of `None`
            (
                ref r @ TypeInfo::ContractCaller {
                    abi_name: ref ran,
                    address: ref ra,
                },
                ref e @ TypeInfo::ContractCaller {
                    abi_name: ref ean,
                    address: ref ea,
                },
            ) => {
                r.eq(e, self.type_engine)
                    || (ran == ean && ra.is_none())
                    || matches!(ran, AbiName::Deferred)
                    || (ran == ean && ea.is_none())
                    || matches!(ean, AbiName::Deferred)
            }

            // this is kinda a hack
            (TypeInfo::ErrorRecovery, _) => true,
            (_, TypeInfo::ErrorRecovery) => true,

            (a, b) => a.eq(&b, self.type_engine),
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
            .map(|x| self.type_engine.look_up_type_id(*x))
            .collect::<Vec<_>>();
        let right_types = right
            .iter()
            .map(|x| self.type_engine.look_up_type_id(*x))
            .collect::<Vec<_>>();
        let mut constraints = vec![];
        for i in 0..(right_types.len() - 1) {
            for j in (i + 1)..right_types.len() {
                let a = right_types.get(i).unwrap();
                let b = right_types.get(j).unwrap();
                if a.eq(b, self.type_engine) {
                    // if a and b are the same type
                    constraints.push((i, j));
                }
            }
        }
        for (i, j) in constraints.into_iter() {
            let a = left_types.get(i).unwrap();
            let b = left_types.get(j).unwrap();
            if matches!(a, TypeInfo::Placeholder(_)) && matches!(b, TypeInfo::Placeholder(_)) {
                continue;
            }
            if !a.eq(b, self.type_engine) {
                return false;
            }
        }

        // if all of the invariants are met, then `self` can be coerced into
        // `other`!
        true
    }
}
