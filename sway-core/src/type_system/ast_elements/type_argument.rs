use crate::{engine_threading::*, language::CallPathTree, type_system::priv_prelude::*};
use std::{cmp::Ordering, fmt, hash::Hasher};
use sway_types::{Span, Spanned};

/// [TypeArgument] can be seen as an "annotated reference" to a [TypeInfo].
/// It holds the [TypeArgument::type_id] which is the actual "reference"
/// to the type, as well as an additional information about that type,
/// called the annotation.
///
/// If a [TypeArgument] only references a [TypeInfo] and is considered as
/// not being annotated, its `initial_type_id` must be the same as `type_id`,
/// its `span` must be [Span::dummy] and its `call_path_tree` must be `None`.
///
/// The annotations are ignored when calculating the [TypeArgument]'s hash
/// (with engines) and equality (with engines).
#[derive(Debug, Clone)]
pub struct TypeArgument {
    /// The [TypeId] of the "referenced" [TypeInfo].
    pub type_id: TypeId,
    /// Denotes the initial type that was referenced before the type
    /// unification, monomorphization, or replacement of [TypeInfo::Custom]s.
    pub initial_type_id: TypeId,
    /// The [Span] related in code to the [TypeInfo] represented by this
    /// [TypeArgument]. This information is mostly used by the LSP and it
    /// differs from use case to use case.
    ///
    /// E.g., in the following example:
    ///
    /// ```ignore
    /// let a: [u64;2] = [0, 0];
    /// let b: [u64;2] = [1, 1];
    /// ```
    ///
    /// the type arguments of the [TypeInfo::Array]s of `a` and `b` will
    /// have two different spans pointing to two different strings "u64".
    /// On the other hand, the two [TypeInfo::Array]s describing the
    /// two instances `[0, 0]`, and `[1, 1]` will have neither the array
    /// type span set, nor the length span, which means they will not be
    /// annotated.
    pub span: Span,
    pub call_path_tree: Option<CallPathTree>,
}

impl TypeArgument {
    /// Returns true if `self` is annotated by having either
    /// its [Self::initial_type_id] different from [Self::type_id],
    /// or [Self::span] different from [Span::dummy]
    /// or [Self::call_path_tree] different from `None`.
    pub fn is_annotated(&self) -> bool {
        self.type_id != self.initial_type_id
            || self.call_path_tree.is_some()
            || !self.span.is_dummy()
    }
}

impl From<TypeId> for TypeArgument {
    /// Creates *a non-annotated* [TypeArgument] that points
    /// to the [TypeInfo] represented by the `type_id`.
    fn from(type_id: TypeId) -> Self {
        TypeArgument {
            type_id,
            initial_type_id: type_id,
            span: Span::dummy(),
            call_path_tree: None,
        }
    }
}

impl Spanned for TypeArgument {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl HashWithEngines for TypeArgument {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TypeArgument {
            type_id,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            initial_type_id: _,
            span: _,
            call_path_tree: _,
        } = self;
        let type_engine = engines.te();
        type_engine.get(*type_id).hash(state, engines);
    }
}

impl EqWithEngines for TypeArgument {}
impl PartialEqWithEngines for TypeArgument {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let type_engine = ctx.engines().te();
        self.type_id == other.type_id
            || type_engine
                .get(self.type_id)
                .eq(&type_engine.get(other.type_id), ctx)
    }
}

impl OrdWithEngines for TypeArgument {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let TypeArgument {
            type_id: lti,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            initial_type_id: _,
            span: _,
            call_path_tree: _,
        } = self;
        let TypeArgument {
            type_id: rti,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            initial_type_id: _,
            span: _,
            call_path_tree: _,
        } = other;
        if lti == rti {
            return Ordering::Equal;
        }
        ctx.engines()
            .te()
            .get(*lti)
            .cmp(&ctx.engines().te().get(*rti), ctx)
    }
}

impl DisplayWithEngines for TypeArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{}", engines.help_out(&*engines.te().get(self.type_id)))
    }
}

impl DebugWithEngines for TypeArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "{:?}",
            engines.help_out(&*engines.te().get(self.type_id))
        )
    }
}

impl From<&TypeParameter> for TypeArgument {
    fn from(type_param: &TypeParameter) -> Self {
        TypeArgument {
            type_id: type_param.type_id,
            initial_type_id: type_param.initial_type_id,
            span: type_param.name.span(),
            call_path_tree: None,
        }
    }
}

impl SubstTypes for TypeArgument {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        self.type_id.subst(ctx)
    }
}
