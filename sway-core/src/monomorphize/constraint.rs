use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use sway_types::Ident;

use crate::{
    decl_engine::DeclId,
    engine_threading::*,
    language::{ty::*, CallPath},
    type_system::*,
};

#[derive(Debug, Clone)]
pub(crate) enum Constraint {
    /// Function declaration/use.
    FnDecl {
        decl_id: DeclId<TyFunctionDecl>,
        subst_list: SubstList,
    },
    /// Struct declaration/use.
    StructDecl {
        decl_id: DeclId<TyStructDecl>,
        subst_list: SubstList,
    },
    /// Enum declaration/use.
    EnumDecl {
        decl_id: DeclId<TyEnumDecl>,
        subst_list: SubstList,
    },
    /// Trait declaration/use.
    TraitDecl {
        decl_id: DeclId<TyTraitDecl>,
        subst_list: SubstList,
    },
    /// Function call.
    FnCall {
        call_path: CallPath,
        decl_id: DeclId<TyFunctionDecl>,
        subst_list: SubstList,
        arguments: Vec<TypeId>,
    },
}

impl Constraint {
    fn discriminant_value(&self) -> u8 {
        use Constraint::*;
        match self {
            FnDecl { .. } => 0,
            StructDecl { .. } => 1,
            EnumDecl { .. } => 2,
            TraitDecl { .. } => 3,
            FnCall { .. } => 4,
        }
    }

    pub(super) fn mk_fn_decl(
        decl_id: &DeclId<TyFunctionDecl>,
        subst_list: &SubstList,
    ) -> Constraint {
        Constraint::FnDecl {
            decl_id: *decl_id,
            subst_list: subst_list.clone(),
        }
    }

    pub(super) fn mk_struct_decl(
        decl_id: &DeclId<TyStructDecl>,
        subst_list: &SubstList,
    ) -> Constraint {
        Constraint::StructDecl {
            decl_id: *decl_id,
            subst_list: subst_list.clone(),
        }
    }

    pub(super) fn mk_enum_decl(decl_id: &DeclId<TyEnumDecl>, subst_list: &SubstList) -> Constraint {
        Constraint::EnumDecl {
            decl_id: *decl_id,
            subst_list: subst_list.clone(),
        }
    }

    pub(super) fn mk_trait_decl(
        decl_id: &DeclId<TyTraitDecl>,
        subst_list: &SubstList,
    ) -> Constraint {
        Constraint::TraitDecl {
            decl_id: *decl_id,
            subst_list: subst_list.clone(),
        }
    }
}

impl From<&TyExpressionVariant> for Constraint {
    fn from(value: &TyExpressionVariant) -> Self {
        match value {
            TyExpressionVariant::FunctionApplication {
                call_path,
                arguments,
                fn_ref,
                ..
            } => Constraint::FnCall {
                call_path: call_path.clone(),
                decl_id: *fn_ref.id(),
                subst_list: fn_ref.subst_list().clone(),
                arguments: args_helper(arguments),
            },
            _ => unimplemented!(),
        }
    }
}

fn args_helper(args: &[(Ident, TyExpression)]) -> Vec<TypeId> {
    args.iter().map(|(_, exp)| exp.return_type).collect()
}

impl EqWithEngines for Constraint {}
impl PartialEqWithEngines for Constraint {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        use Constraint::*;
        let type_engine = engines.te();
        match (self, other) {
            (
                FnDecl {
                    decl_id: ldi,
                    subst_list: lsl,
                },
                FnDecl {
                    decl_id: rdi,
                    subst_list: rsl,
                },
            ) => ldi == rdi && lsl.eq(rsl, engines),
            (
                StructDecl {
                    decl_id: ldi,
                    subst_list: lsl,
                },
                StructDecl {
                    decl_id: rdi,
                    subst_list: rsl,
                },
            ) => ldi == rdi && lsl.eq(rsl, engines),
            (
                EnumDecl {
                    decl_id: ldi,
                    subst_list: lsl,
                },
                EnumDecl {
                    decl_id: rdi,
                    subst_list: rsl,
                },
            ) => ldi == rdi && lsl.eq(rsl, engines),
            (
                TraitDecl {
                    decl_id: ldi,
                    subst_list: lsl,
                },
                TraitDecl {
                    decl_id: rdi,
                    subst_list: rsl,
                },
            ) => ldi == rdi && lsl.eq(rsl, engines),
            (
                FnCall {
                    call_path: lcp,
                    decl_id: ldi,
                    subst_list: lsl,
                    arguments: la,
                },
                FnCall {
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
        use Constraint::*;
        let type_engine = engines.te();
        match self {
            FnDecl {
                decl_id,
                subst_list,
            } => {
                decl_id.hash(state);
                subst_list.hash(state, engines);
            }
            StructDecl {
                decl_id,
                subst_list,
            } => {
                decl_id.hash(state);
                subst_list.hash(state, engines);
            }
            EnumDecl {
                decl_id,
                subst_list,
            } => {
                decl_id.hash(state);
                subst_list.hash(state, engines);
            }
            TraitDecl {
                decl_id,
                subst_list,
            } => {
                decl_id.hash(state);
                subst_list.hash(state, engines);
            }
            FnCall {
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
        use Constraint::*;
        let type_engine = engines.te();
        match (self, other) {
            (
                FnDecl {
                    decl_id: ldi,
                    subst_list: lsl,
                },
                FnDecl {
                    decl_id: rdi,
                    subst_list: rsl,
                },
            ) => ldi.cmp(rdi).then_with(|| lsl.cmp(rsl, engines)),
            (
                StructDecl {
                    decl_id: ldi,
                    subst_list: lsl,
                },
                StructDecl {
                    decl_id: rdi,
                    subst_list: rsl,
                },
            ) => ldi.cmp(rdi).then_with(|| lsl.cmp(rsl, engines)),
            (
                EnumDecl {
                    decl_id: ldi,
                    subst_list: lsl,
                },
                EnumDecl {
                    decl_id: rdi,
                    subst_list: rsl,
                },
            ) => ldi.cmp(rdi).then_with(|| lsl.cmp(rsl, engines)),
            (
                TraitDecl {
                    decl_id: ldi,
                    subst_list: lsl,
                },
                TraitDecl {
                    decl_id: rdi,
                    subst_list: rsl,
                },
            ) => ldi.cmp(rdi).then_with(|| lsl.cmp(rsl, engines)),
            (
                FnCall {
                    call_path: lcp,
                    decl_id: ldi,
                    subst_list: lsl,
                    arguments: la,
                },
                FnCall {
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
