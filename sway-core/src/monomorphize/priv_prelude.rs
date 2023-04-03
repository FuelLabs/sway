use sway_types::Ident;

pub(super) type PathBuf = Vec<Ident>;

pub(super) use super::{
    constraint::*,
    // flatten::{
    //     code_block::flatten_code_block,
    //     declaration::flatten_decl,
    //     expression::flatten_exp,
    //     flatten_ast,
    //     module::{flatten_module, flatten_root},
    //     node::flatten_node,
    // },
    gather::{
        code_block::gather_from_code_block,
        context::{GatherContext, GatherNamespace},
        declaration::gather_from_decl,
        expression::gather_from_exp,
        gather_constraints,
        module::gather_from_root,
        node::gather_from_node,
        type_system::{gather_from_trait_constraints, gather_from_ty},
    },
    instruct::{
        apply_instructions,
        code_block::instruct_code_block,
        context::{InstructContext, InstructionItems},
        declaration::instruct_decl,
        expression::instruct_exp,
        module::instruct_root,
        node::instruct_node,
    },
    instructions::Instruction,
    solve::{
        instruction_result::InstructionResult, iteration_report::IterationReport, solver::Solver,
        ConstraintPQ, ConstraintTick, ConstraintWrapper,
    },
};
