use std::sync::RwLock;

use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    decl_engine::*, engine_threading::*, language::ty::*, monomorphize::priv_prelude::*,
    type_system::*,
};

/// Contextual state tracked and accumulated throughout solving [Constraint]s.
pub(crate) struct Solver<'a> {
    /// The type engine storing types.
    type_engine: &'a TypeEngine,

    /// The declaration engine holds declarations.
    decl_engine: &'a DeclEngine,

    /// The instructions returned by the [Solver].
    instructions: Vec<Instruction>,

    state_graphs: &'a RwLock<StateGraphs>,
}

impl<'a> Solver<'a> {
    /// Creates a new [Solver].
    pub(crate) fn new(engines: Engines<'a>, state_graphs: &'a RwLock<StateGraphs>) -> Solver<'a> {
        let (type_engine, decl_engine) = engines.unwrap();
        Solver {
            type_engine,
            decl_engine,
            instructions: vec![],
            state_graphs,
        }
    }

    /// Takes a [Solver] and returns the list of [Instruction]s from that
    /// [Solver].
    pub(crate) fn into_instructions(self) -> Vec<Instruction> {
        self.instructions
    }

    fn wrap_constraint(&self, constraint: Constraint, num_times: usize) -> ConstraintWrapper {
        WithEngines {
            thing: ConstraintTick::new(constraint, num_times),
            engines: Engines::new(self.type_engine, self.decl_engine),
        }
    }

    /// Solves a set of constraints with a given [Solver].
    pub(crate) fn solve<T>(&mut self, handler: &Handler, constraints: T) -> Result<(), ErrorEmitted>
    where
        T: IntoIterator<Item = Constraint>,
    {
        let mut constraints: ConstraintPQ = constraints
            .into_iter()
            .map(|c| self.wrap_constraint(c, 0))
            .collect();

        // for constraint in constraints.iter() {
        //     println!("{:#?}", constraint);
        // }

        let mut iterations = 0;
        let mut instructions = vec![];

        while iterations < 100 && !constraints.is_empty() {
            let report = self.helper(
                handler,
                constraints
                    .into_sorted_vec()
                    .into_iter()
                    .map(|c| c.thing)
                    .collect(),
            )?;

            constraints = report.new_constraints;
            instructions.extend(report.instructions);

            iterations += 1;
        }

        self.instructions.extend(instructions);

        Ok(())
    }

    fn helper(
        &self,
        handler: &Handler,
        constraints: Vec<ConstraintTick>,
    ) -> Result<IterationReport, ErrorEmitted> {
        let mut new_constraints = ConstraintPQ::new();
        let mut instructions = vec![];

        for constraint_tick in constraints.into_iter() {
            let ConstraintTick {
                constraint,
                num_times,
            } = constraint_tick;
            let instruction_res = match &constraint {
                Constraint::FnDecl {
                    decl_id,
                    subst_list,
                } => self.helper_fn_decl(decl_id, subst_list),
                Constraint::StructDecl {
                    decl_id,
                    subst_list,
                } => self.helper_struct_decl(decl_id, subst_list),
                Constraint::EnumDecl {
                    decl_id,
                    subst_list,
                } => self.helper_enum_decl(decl_id, subst_list),
                Constraint::TraitDecl {
                    decl_id,
                    subst_list,
                } => self.helper_trait_decl(decl_id, subst_list),
                Constraint::FnCall {
                    decl_id,
                    subst_list,
                    arguments,
                    ..
                } => todo!(),
            };
            match instruction_res {
                InstructionResult::NewInstructions(instruction_res) => {
                    instructions.extend(instruction_res);
                }
                InstructionResult::RedoConstraint => {
                    new_constraints.push(self.wrap_constraint(constraint, num_times + 1));
                }
            }
        }

        let report = IterationReport {
            new_constraints,
            instructions,
        };

        Ok(report)
    }

    fn helper_fn_decl(
        &self,
        fn_id: &DeclId<TyFunctionDecl>,
        subst_list: &SubstList,
    ) -> InstructionResult {
        todo!()
        // self.helper_subst_list(subst_list).and(|| {
        //     let engines = Engines::new(self.type_engine, self.decl_engine);
        //     let fn_ref = self.decl_engine.insert(
        //         self.decl_engine
        //             .get_function(fn_id)
        //             .subst(engines, subst_list)
        //             .into_inner(),
        //         SubstList::new(),
        //     );
        //     let mut fn_graph = self.fn_graph.write().unwrap();
        //     let parent = fn_graph.add_node(*fn_id);
        //     let next = fn_graph.add_node(*fn_ref.id());
        //     fn_graph.add_edge(parent, next, None);
        //     InstructionResult::empty()
        // })
    }

    fn helper_struct_decl(
        &self,
        struct_id: &DeclId<TyStructDecl>,
        subst_list: &SubstList,
    ) -> InstructionResult {
        todo!()
        // self.helper_subst_list(subst_list).and(|| {
        //     let engines = Engines::new(self.type_engine, self.decl_engine);
        //     let struct_ref = self.decl_engine.insert(
        //         self.decl_engine
        //             .get_struct(struct_id)
        //             .subst(engines, subst_list)
        //             .into_inner(),
        //         SubstList::new(),
        //     );
        //     let mut struct_graph = self.struct_graph.write().unwrap();
        //     let parent = struct_graph.add_node(*struct_id);
        //     let next = struct_graph.add_node(*struct_ref.id());
        //     struct_graph.add_edge(parent, next, None);
        //     InstructionResult::empty()
        // })
    }

    fn helper_enum_decl(
        &self,
        enum_id: &DeclId<TyEnumDecl>,
        subst_list: &SubstList,
    ) -> InstructionResult {
        todo!()
        // self.helper_subst_list(subst_list).and(|| {
        //     let engines = Engines::new(self.type_engine, self.decl_engine);
        //     let enum_ref = self.decl_engine.insert(
        //         self.decl_engine
        //             .get_enum(enum_id)
        //             .subst(engines, subst_list)
        //             .into_inner(),
        //         SubstList::new(),
        //     );
        //     let mut enum_graph = self.enum_graph.write().unwrap();
        //     let parent = enum_graph.add_node(*enum_id);
        //     let next = enum_graph.add_node(*enum_ref.id());
        //     enum_graph.add_edge(parent, next, None);
        //     InstructionResult::empty()
        // })
    }

    fn helper_trait_decl(
        &self,
        trait_id: &DeclId<TyTraitDecl>,
        subst_list: &SubstList,
    ) -> InstructionResult {
        todo!()
        // self.helper_subst_list(subst_list).and(|| {
        //     let engines = Engines::new(self.type_engine, self.decl_engine);
        //     let trait_ref = self.decl_engine.insert(
        //         self.decl_engine
        //             .get_trait(trait_id)
        //             .subst(engines, subst_list)
        //             .into_inner(),
        //         SubstList::new(),
        //     );
        //     let mut trait_graph = self.trait_graph.write().unwrap();
        //     let parent = trait_graph.add_node(*trait_id);
        //     let next = trait_graph.add_node(*trait_ref.id());
        //     trait_graph.add_edge(parent, next, None);
        //     InstructionResult::empty()
        // })
    }

    fn helper_subst_list(&self, subst_list: &SubstList) -> InstructionResult {
        subst_list
            .elems()
            .into_iter()
            .map(|type_param| self.helper_ty_use(type_param.type_id))
            .collect()
    }

    fn helper_ty_use(&self, type_id: TypeId) -> InstructionResult {
        use TypeInfo::*;
        match self.type_engine.get(type_id) {
            TypeParam { .. } => InstructionResult::redo(),
            Unknown => todo!(),
            UnknownGeneric {
                name,
                trait_constraints,
            } => todo!(),
            Placeholder(_) => todo!(),
            Enum(_) => todo!(),
            Struct(_) => todo!(),
            Tuple(_) => todo!(),
            ContractCaller { abi_name, address } => todo!(),
            Custom {
                call_path,
                type_arguments,
            } => todo!(),
            Array(_, _) => todo!(),
            Storage { fields } => todo!(),
            Alias { name, ty } => todo!(),
            Str(_) | UnsignedInteger(_) | Boolean | SelfType | B256 | Numeric | Contract
            | ErrorRecovery | RawUntypedPtr | RawUntypedSlice => InstructionResult::empty(),
        }
    }

    // fn helper_ty_use(&self, type_id: TypeId) -> Result<InstructionResult, ErrorEmitted> {
    //     let mut instructions = vec![];

    //     match self.type_engine.get(type_id) {
    //         TypeInfo::Unknown => todo!(),
    //         TypeInfo::UnknownGeneric { .. } => todo!(),
    //         TypeInfo::Placeholder(_) => todo!(),
    //         TypeInfo::TypeParam { .. } => todo!(),
    //         TypeInfo::Enum { .. } => todo!(),
    //         TypeInfo::Struct { .. } => todo!(),
    //         TypeInfo::Tuple(elems) => {
    //             let res: InstructionResult = elems
    //                 .into_iter()
    //                 .map(|type_arg| self.helper_ty_use(type_arg.type_id))
    //                 .collect::<Result<_, _>>()?;
    //             match res {
    //                 InstructionResult::NewInstructions(new_instructions) => {
    //                     instructions.extend(new_instructions);
    //                 }
    //                 InstructionResult::NoInstruction => {}
    //                 InstructionResult::RedoConstraint => {
    //                     return Ok(InstructionResult::RedoConstraint);
    //                 }
    //             }
    //         }
    //         TypeInfo::ContractCaller { .. } => todo!(),
    //         TypeInfo::Custom { .. } => todo!(),
    //         TypeInfo::SelfType => todo!(),
    //         TypeInfo::Numeric => todo!(),
    //         TypeInfo::ErrorRecovery => todo!(),
    //         TypeInfo::Array(_, _) => todo!(),
    //         TypeInfo::Storage { .. } => todo!(),
    //         TypeInfo::Alias { .. } => todo!(),
    //         TypeInfo::Str(_)
    //         | TypeInfo::UnsignedInteger(_)
    //         | TypeInfo::Boolean
    //         | TypeInfo::B256
    //         | TypeInfo::Contract
    //         | TypeInfo::RawUntypedPtr
    //         | TypeInfo::RawUntypedSlice => {}
    //     }

    //     Ok(InstructionResult::from_instructions(instructions))
    // }

    // fn helper_fn_call(
    //     &self,
    //     _handler: &Handler,
    //     decl_id: DeclId<ty::TyFunctionDecl>,
    //     subst_list: SubstList,
    //     _arguments: Vec<TypeId>,
    // ) -> Result<InstructionResult, ErrorEmitted> {
    //     let mut instructions = vec![];

    //     let res: InstructionResult = subst_list
    //         .elems()
    //         .iter()
    //         .map(|type_param| self.helper_ty_use(type_param.type_id))
    //         .collect::<Result<_, _>>()?;
    //     match res {
    //         InstructionResult::NewInstructions(new_instructions) => {
    //             instructions.extend(new_instructions);
    //         }
    //         InstructionResult::NoInstruction => {}
    //         InstructionResult::RedoConstraint => {
    //             return Ok(InstructionResult::RedoConstraint);
    //         }
    //     }

    //     if !subst_list.is_empty() {
    //         instructions.push(Instruction::FnDecl(decl_id, subst_list));
    //     }

    //     Ok(InstructionResult::from_instructions(instructions))
    // }
}
