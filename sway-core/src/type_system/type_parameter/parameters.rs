use crate::{engine_threading::*, error::*, semantic_analysis::*, type_system::*};

use sway_error::error::CompileError;

use std::{
    cmp::Ordering,
    hash::Hasher,
    slice::{Iter, IterMut},
};

/// Container type representing a list of type parameters.
#[derive(Debug, Clone, Default)]
pub struct TypeParameters {
    /// The "self type" that encapsulated by this scope, if any.
    self_type: Option<TypeParameter>,

    /// List of [TypeParameter]s in this scope.
    list: Vec<TypeParameter>,
}

impl TypeParameters {
    /// Creates a new [TypeParameters].
    pub fn new() -> TypeParameters {
        TypeParameters {
            self_type: None,
            list: vec![],
        }
    }

    /// Creates a new [TypeParameters] given a `self_type`.
    pub fn new_with_self_type(self_type: Option<TypeParameter>) -> TypeParameters {
        TypeParameters {
            self_type,
            list: vec![],
        }
    }

    /// Creates a new [TypeParameters], where the new [TypeParameters] contains
    /// one element: the "self type" from `self`.
    pub(crate) fn drop_everything_but_self(self) -> TypeParameters {
        TypeParameters {
            self_type: self.self_type,
            list: vec![],
        }
    }

    /// Returns the "self type", if it exists.
    pub(crate) fn get_self_type(&self) -> Option<&TypeParameter> {
        self.self_type.as_ref()
    }

    /// Returns a mutable reference to the "self type".
    pub fn get_self_type_mut(&mut self) -> Option<&mut TypeParameter> {
        self.self_type.as_mut()
    }

    /// Returns the [TypeParameter]s from `self` as a slice.
    /// Excludes any [TypeParameter] that might exist for a "self type".
    pub fn as_slice(&self) -> &[TypeParameter] {
        &self.list
    }

    /// Returns `true` if `self` contains 0 [TypeParameter].
    /// Excludes any [TypeParameter] that might exist for a "self type".
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Returns the number of [TypeParameter] in `self`.
    /// Excludes any [TypeParameter] that might exist for a "self type".
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Extends this [TypeParameters] with another [TypeParameters].
    /// Excludes any [TypeParameter] that might exist for a "self type" in
    /// `other`.
    pub(crate) fn extend(&mut self, other: TypeParameters) {
        self.list.extend(other.list);
    }

    /// Iterates immutably through the [TypeParameter]s in `self`.
    /// Excludes any [TypeParameter] that might exist for a "self type".
    pub fn iter(&self) -> Iter<'_, TypeParameter> {
        self.list.iter()
    }

    /// Iterates mutably through the [TypeParameter]s in `self`.
    /// Excludes any [TypeParameter] that might exist for a "self type".
    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, TypeParameter> {
        self.list.iter_mut()
    }

    /// Iterates immutably through the [TypeParameter]s in `self`.
    pub fn iter_including_self(&self) -> TypeParametersIter<'_> {
        TypeParametersIter::new(&self.self_type, false, self.list.iter())
    }

    /// Type check a [TypeParameters] and return a new [TypeParameters]. This
    /// will also insert this new list into the current namespace.
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        type_params: Vec<TypeParameter>,
        disallow_trait_constraints: bool,
        self_type_param: Option<TypeParameter>,
    ) -> CompileResult<TypeParameters> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut new_type_params: Vec<TypeParameter> = vec![];

        if let Some(self_type_param) = self_type_param.clone() {
            self_type_param.insert_self_type_into_namespace(ctx.by_ref());
        }

        for type_param in type_params.into_iter() {
            if disallow_trait_constraints && !type_param.trait_constraints.is_empty() {
                let errors = vec![CompileError::WhereClauseNotYetSupported {
                    span: type_param.trait_constraints_span,
                }];
                return err(vec![], errors);
            }
            new_type_params.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_param),
                continue,
                warnings,
                errors
            ));
        }

        if errors.is_empty() {
            ok(
                TypeParameters {
                    self_type: self_type_param,
                    list: new_type_params,
                },
                warnings,
                errors,
            )
        } else {
            err(warnings, errors)
        }
    }
}

impl From<Vec<TypeParameter>> for TypeParameters {
    fn from(value: Vec<TypeParameter>) -> Self {
        TypeParameters {
            self_type: None,
            list: value,
        }
    }
}

impl FromIterator<TypeParameter> for TypeParameters {
    fn from_iter<I: IntoIterator<Item = TypeParameter>>(iter: I) -> Self {
        TypeParameters {
            self_type: None,
            list: iter.into_iter().collect(),
        }
    }
}

impl HashWithEngines for TypeParameters {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TypeParameters { self_type, list } = self;
        self_type.hash(state, engines);
        list.hash(state, engines);
    }
}

impl EqWithEngines for TypeParameters {}
impl PartialEqWithEngines for TypeParameters {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let TypeParameters {
            self_type: lst,
            list: ll,
        } = self;
        let TypeParameters {
            self_type: rst,
            list: rl,
        } = other;
        lst.eq(rst, engines) && ll.eq(rl, engines)
    }
}

impl OrdWithEngines for TypeParameters {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> Ordering {
        let TypeParameters {
            self_type: lst,
            list: ll,
        } = self;
        let TypeParameters {
            self_type: rst,
            list: rl,
        } = other;
        lst.cmp(rst, engines).then_with(|| ll.cmp(rl, engines))
    }
}

impl SubstTypes for TypeParameters {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        if let Some(type_param) = self.self_type.as_mut() {
            type_param.subst(type_mapping, engines);
        }
        self.list
            .iter_mut()
            .for_each(|type_param| type_param.subst(type_mapping, engines));
    }
}
