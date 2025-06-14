use crate::{decl_engine::DeclEngineGet as _, engine_threading::*, language::CallPathTree, type_system::priv_prelude::*};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt, hash::Hasher};
use sway_types::{Span, Spanned};

use super::type_parameter::ConstGenericExpr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericTypeArgument {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericConstArgument {
    pub expr: ConstGenericExpr,
}

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericArgument {
    Type(GenericTypeArgument),
    Const(GenericConstArgument),
}

impl GenericArgument {
    pub fn as_type_argument(&self) -> Option<&GenericTypeArgument> {
        match self {
            GenericArgument::Type(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_type_argument_mut(&mut self) -> Option<&mut GenericTypeArgument> {
        match self {
            GenericArgument::Type(a) => Some(a),
            _ => None,
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.as_type_argument()
            .expect("only works with type arguments")
            .type_id
    }

    pub fn type_id_mut(&mut self) -> &mut TypeId {
        &mut self
            .as_type_argument_mut()
            .expect("only works with type arguments")
            .type_id
    }

    pub fn initial_type_id(&self) -> TypeId {
        self.as_type_argument()
            .expect("only works with type arguments")
            .initial_type_id
    }

    pub fn call_path_tree(&self) -> Option<&CallPathTree> {
        match self {
            GenericArgument::Type(a) => a.call_path_tree.as_ref(),
            GenericArgument::Const(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }

    pub fn call_path_tree_mut(&mut self) -> Option<&mut CallPathTree> {
        match self {
            GenericArgument::Type(a) => a.call_path_tree.as_mut(),
            GenericArgument::Const(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }

    /// Returns true if `self` is annotated by having either
    /// its [Self::initial_type_id] different from [Self::type_id],
    /// or [Self::span] different from [Span::dummy]
    /// or [Self::call_path_tree] different from `None`.
    pub fn is_annotated(&self) -> bool {
        match self {
            GenericArgument::Type(a) => {
                a.type_id != a.initial_type_id || a.call_path_tree.is_some() || !a.span.is_dummy()
            }
            GenericArgument::Const(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }
}

impl Spanned for GenericArgument {
    fn span(&self) -> Span {
        match self {
            GenericArgument::Type(a) => a.span.clone(),
            GenericArgument::Const(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }
}

impl From<TypeId> for GenericArgument {
    /// Creates *a non-annotated* [TypeArgument] that points
    /// to the [TypeInfo] represented by the `type_id`.
    fn from(type_id: TypeId) -> Self {
        GenericArgument::Type(GenericTypeArgument {
            type_id,
            initial_type_id: type_id,
            span: Span::dummy(),
            call_path_tree: None,
        })
    }
}

impl HashWithEngines for GenericArgument {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        match self {
            GenericArgument::Type(GenericTypeArgument {
                type_id,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                initial_type_id: _,
                span: _,
                call_path_tree: _,
            }) => {
                let type_engine = engines.te();
                type_engine.get(*type_id).hash(state, engines);
            }
            GenericArgument::Const(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }
}

impl EqWithEngines for GenericArgument {}
impl PartialEqWithEngines for GenericArgument {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (GenericArgument::Type(l), GenericArgument::Type(r)) => {
                let type_engine = ctx.engines().te();
                l.type_id == r.type_id
                    || type_engine
                        .get(l.type_id)
                        .eq(&type_engine.get(r.type_id), ctx)
            }
            (GenericArgument::Const(_), GenericArgument::Const(_)) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
            _ => false,
        }
    }
}

impl OrdWithEngines for GenericArgument {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        match (self, other) {
            (
                GenericArgument::Type(GenericTypeArgument {
                    type_id: lti,
                    // these fields are not compared because they aren't relevant/a
                    // reliable source of obj v. obj distinction
                    initial_type_id: _,
                    span: _,
                    call_path_tree: _,
                }),
                GenericArgument::Type(GenericTypeArgument {
                    type_id: rti,
                    // these fields are not compared because they aren't relevant/a
                    // reliable source of obj v. obj distinction
                    initial_type_id: _,
                    span: _,
                    call_path_tree: _,
                }),
            ) => {
                if lti == rti {
                    return Ordering::Equal;
                }
                ctx.engines()
                    .te()
                    .get(*lti)
                    .cmp(&ctx.engines().te().get(*rti), ctx)
            }
            (GenericArgument::Const(_), GenericArgument::Const(_)) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
            (GenericArgument::Type(_), _) => Ordering::Less,
            (GenericArgument::Const(_), _) => Ordering::Greater,
        }
    }
}

impl DisplayWithEngines for GenericArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        match self {
            GenericArgument::Type(a) => {
                write!(f, "{}", engines.help_out(&*engines.te().get(a.type_id)))
            }
            GenericArgument::Const(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }
}

impl DebugWithEngines for GenericArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        match self {
            GenericArgument::Type(a) => {
                write!(f, "{:?}", engines.help_out(&*engines.te().get(a.type_id)))
            }
            GenericArgument::Const(_) => {
                todo!("Will be implemented by https://github.com/FuelLabs/sway/issues/6860")
            }
        }
    }
}


impl GenericArgument {
    pub fn from(engines: &Engines, p: &TypeParameter) -> Self {
        match p {
            TypeParameter::Type(p) => GenericArgument::Type(GenericTypeArgument {
                type_id: p.type_id,
                initial_type_id: p.initial_type_id,
                span: p.name.span(),
                call_path_tree: None,
            }),
            TypeParameter::Const(p) => {
                let decl = engines.de().get(p.decl_ref.id());
                GenericArgument::Const(GenericConstArgument {
                    expr: match decl.value.as_ref() {
                        Some(expr) => expr.clone(),
                        None => ConstGenericExpr::AmbiguousVariableExpression {
                            ident: decl.name().clone(),
                        },
                    },
                })
            },
        }
    }
}

impl SubstTypes for GenericArgument {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        match self {
            GenericArgument::Type(a) => a.type_id.subst(ctx),
            GenericArgument::Const(_) => HasChanges::No,
        }
    }
}
