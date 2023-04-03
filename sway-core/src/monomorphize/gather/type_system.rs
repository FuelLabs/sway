use crate::{monomorphize::priv_prelude::*, type_system::*};

pub(crate) fn gather_from_ty(mut ctx: GatherContext, type_id: TypeId) {
    use TypeInfo::*;
    let type_engine = ctx.type_engine;
    match type_engine.get(type_id) {
        UnknownGeneric {
            name: _,
            trait_constraints,
        } => gather_from_trait_constraints(ctx, trait_constraints.0),
        Enum(enum_ref) => {
            ctx.add_constraint(Constraint::mk_enum_decl(
                enum_ref.id(),
                enum_ref.subst_list(),
            ));
        }
        Struct(struct_ref) => {
            ctx.add_constraint(Constraint::mk_struct_decl(
                struct_ref.id(),
                struct_ref.subst_list(),
            ));
        }
        Tuple(elems) => {
            for elem in elems {
                gather_from_ty(ctx.by_ref(), elem.type_id);
            }
        }
        ContractCaller {
            abi_name: _,
            address,
        } => {
            if let Some(address) = address {
                gather_from_exp(ctx, &address);
            }
        }
        Custom {
            call_path: _,
            type_arguments,
        } => {
            if let Some(type_args) = type_arguments {
                for type_arg in type_args {
                    gather_from_ty(ctx.by_ref(), type_arg.type_id);
                }
            }
        }
        Array(ty, _) => {
            gather_from_ty(ctx.by_ref(), ty.type_id);
        }
        Storage { fields } => {
            for field in fields {
                gather_from_ty(ctx.by_ref(), field.type_argument.type_id);
            }
        }
        Alias { name: _, ty } => {
            gather_from_ty(ctx.by_ref(), ty.type_id);
        }
        Unknown
        | Placeholder(_)
        | TypeParam { .. }
        | Str(_)
        | UnsignedInteger(_)
        | Boolean
        | SelfType
        | B256
        | Numeric
        | Contract
        | ErrorRecovery
        | RawUntypedPtr
        | RawUntypedSlice => {}
    }
}

pub(crate) fn gather_from_trait_constraints(
    mut ctx: GatherContext,
    constraints: Vec<TraitConstraint>,
) {
    for c in constraints {
        let TraitConstraint {
            trait_name: _,
            type_arguments,
        } = c;
        for type_arg in type_arguments {
            gather_from_ty(ctx.by_ref(), type_arg.type_id);
        }
    }
}
