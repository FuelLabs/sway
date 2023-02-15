use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use sway_types::Ident;

use crate::{
    decl_engine::DeclId,
    engine_threading::*,
    language::{ty, CallPath},
    type_system::*,
};

#[derive(Debug, Clone)]
pub(crate) enum Constraint {
    /// Type use.
    Ty(TypeId),
    /// Function call.
    FnCall {
        call_path: CallPath,
        decl_id: DeclId<ty::TyFunctionDeclaration>,
        subst_list: TypeSubstList,
        arguments: Vec<TypeId>,
    },
}

impl Constraint {
    fn discriminant_value(&self) -> u8 {
        match self {
            Constraint::Ty(_) => 0,
            Constraint::FnCall { .. } => 1,
        }
    }
}

impl From<&TypeId> for Constraint {
    fn from(value: &TypeId) -> Self {
        Constraint::Ty(*value)
    }
}

impl From<TypeId> for Constraint {
    fn from(value: TypeId) -> Self {
        Constraint::Ty(value)
    }
}

impl From<&ty::TyExpressionVariant> for Constraint {
    fn from(value: &ty::TyExpressionVariant) -> Self {
        match value {
            ty::TyExpressionVariant::FunctionApplication {
                call_path,
                arguments,
                function_decl_ref,
                ..
            } => Constraint::FnCall {
                call_path: call_path.clone(),
                decl_id: *function_decl_ref.id(),
                subst_list: function_decl_ref.subst_list().clone(),
                arguments: args_helper(arguments),
            },
            _ => unimplemented!(),
        }
    }
}

fn args_helper(args: &[(Ident, ty::TyExpression)]) -> Vec<TypeId> {
    args.iter().map(|(_, exp)| exp.return_type).collect()
}

impl EqWithEngines for Constraint {}
impl PartialEqWithEngines for Constraint {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        match (self, other) {
            (Constraint::Ty(l), Constraint::Ty(r)) => {
                type_engine.get(*l).eq(&type_engine.get(*r), engines)
            }
            (
                Constraint::FnCall {
                    call_path: lcp,
                    decl_id: ldi,
                    subst_list: lsl,
                    arguments: la,
                },
                Constraint::FnCall {
                    call_path: rcp,
                    decl_id: rdi,
                    subst_list: rsl,
                    arguments: ra,
                },
            ) => {
                lcp == rcp
                    && ldi == rdi
                    && lsl.eq(rsl, engines)
                    && la.len() == ra.len()
                    && la
                        .iter()
                        .zip(ra.iter())
                        .map(|(l, r)| type_engine.get(*l).eq(&type_engine.get(*r), engines))
                        .all(|b| b)
            }
            _ => false,
        }
    }
}

impl HashWithEngines for Constraint {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let type_engine = engines.te();
        match self {
            Constraint::Ty(type_id) => {
                state.write_u8(self.discriminant_value());
                type_engine.get(*type_id).hash(state, engines);
            }
            Constraint::FnCall {
                call_path,
                decl_id,
                subst_list,
                arguments,
            } => {
                state.write_u8(self.discriminant_value());
                call_path.hash(state);
                decl_id.hash(state);
                subst_list.hash(state, engines);
                arguments
                    .iter()
                    .for_each(|arg| type_engine.get(*arg).hash(state, engines));
            }
        }
    }
}

impl OrdWithEngines for Constraint {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> Ordering {
        let type_engine = engines.te();
        match (self, other) {
            (Constraint::Ty(l), Constraint::Ty(r)) => {
                type_engine.get(*l).cmp(&type_engine.get(*r), engines)
            }
            (
                Constraint::FnCall {
                    call_path: lcp,
                    decl_id: ldi,
                    subst_list: lsl,
                    arguments: la,
                },
                Constraint::FnCall {
                    call_path: rcp,
                    decl_id: rdi,
                    subst_list: rsl,
                    arguments: ra,
                },
            ) => lcp
                .cmp(rcp)
                .then_with(|| ldi.cmp(rdi))
                .then_with(|| lsl.cmp(rsl, engines))
                .then_with(|| la.len().cmp(&ra.len()))
                .then_with(|| {
                    la.iter()
                        .zip(ra.iter())
                        .fold(Ordering::Equal, |acc, (l, r)| {
                            acc.then_with(|| type_engine.get(*l).cmp(&type_engine.get(*r), engines))
                        })
                }),
            (l, r) => l.discriminant_value().cmp(&r.discriminant_value()),
        }
    }
}
