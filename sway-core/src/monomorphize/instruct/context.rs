use std::sync::RwLock;

use hashbrown::HashMap;

use crate::{decl_engine::*, language::ty, monomorphize::priv_prelude::*, Engines};

/// Contextual state tracked and accumulated throughout applying the
/// monomorphization instructions.
pub(crate) struct InstructContext<'a> {
    pub(crate) engines: Engines,

    /// All of the instructions, sorted.
    instructions: &'a RwLock<InstructionItems>,
}

impl<'a> InstructContext<'a> {
    /// Initialize a context at the top-level of a module with its namespace.
    pub(crate) fn from_root(engines: &Engines, instructions: &'a RwLock<InstructionItems>) -> Self {
        Self::from_module_namespace(engines, instructions)
    }

    fn from_module_namespace(
        engines: &Engines,
        instructions: &'a RwLock<InstructionItems>,
    ) -> Self {
        Self {
            engines: engines.clone(),
            instructions,
        }
    }

    /// Create a new context that mutably borrows the inner [Namespace] with a
    /// lifetime bound by `self`.
    pub(crate) fn by_ref(&mut self) -> InstructContext<'_> {
        InstructContext {
            engines: self.engines.clone(),
            instructions: self.instructions,
        }
    }

    /// Scope the [InstructContext] with the given [Namespace].
    pub(crate) fn scoped(self) -> InstructContext<'a> {
        InstructContext {
            engines: self.engines.clone(),
            instructions: self.instructions,
        }
    }
}

type FnMap = HashMap<DeclId<ty::TyFunctionDecl>, Vec<Instruction>>;
type TraitMap = HashMap<DeclId<ty::TyTraitDecl>, Vec<Instruction>>;
type ImplTraitMap = HashMap<DeclId<ty::TyImplTrait>, Vec<Instruction>>;
type StructMap = HashMap<DeclId<ty::TyStructDecl>, Vec<Instruction>>;
type EnumMap = HashMap<DeclId<ty::TyEnumDecl>, Vec<Instruction>>;

pub(crate) struct InstructionItems {
    /// A map of [TyFunctionDecl](ty::TyFunctionDecl) [DeclId]s to be
    /// monomorphized.
    fn_map: FnMap,

    /// A map of [TyTraitDecl](ty::TyTraitDecl) [DeclId]s to be monomorphized.
    trait_map: TraitMap,

    /// A map of [TyImplTrait](ty::TyImplTrait) [DeclId]s to be monomorphized.
    impl_trait_map: ImplTraitMap,

    /// A map of [TyStructDecl](ty::TyStructDecl) [DeclId]s to be monomorphized.
    struct_map: StructMap,

    /// A map of [TyEnumDecl](ty::TyEnumDecl) [DeclId]s to be monomorphized.
    enum_map: EnumMap,

    /// The list of instructions not included in any of the previous fields.
    instructions: Vec<Instruction>,
}

impl InstructionItems {
    pub(crate) fn new(instructions: Vec<Instruction>) -> InstructionItems {
        let mut fn_map: FnMap = HashMap::new();
        let mut trait_map: TraitMap = HashMap::new();
        let mut impl_trait_map: ImplTraitMap = HashMap::new();
        let mut struct_map: StructMap = HashMap::new();
        let mut enum_map: EnumMap = HashMap::new();
        let mut leftovers = vec![];
        for instruction in instructions.into_iter() {
            match &instruction {
                Instruction::FnDecl(decl_id, _) => {
                    let v = fn_map.entry(*decl_id).or_default();
                    v.push(instruction);
                }
                Instruction::TraitDecl(decl_id, _) => {
                    let v = trait_map.entry(*decl_id).or_default();
                    v.push(instruction);
                }
                Instruction::ImplTrait(decl_id, _) => {
                    let v = impl_trait_map.entry(*decl_id).or_default();
                    v.push(instruction);
                }
                Instruction::StructDecl(decl_id, _) => {
                    let v = struct_map.entry(*decl_id).or_default();
                    v.push(instruction);
                }
                Instruction::EnumDecl(decl_id, _) => {
                    let v = enum_map.entry(*decl_id).or_default();
                    v.push(instruction);
                }
                _ => {
                    leftovers.push(instruction);
                }
            }
        }

        InstructionItems {
            fn_map,
            trait_map,
            impl_trait_map,
            struct_map,
            enum_map,
            instructions: leftovers,
        }
    }
}
