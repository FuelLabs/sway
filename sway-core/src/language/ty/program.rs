use fuel_tx::StorageSlot;
use sway_types::Ident;

use crate::{
    language::{parsed, ty::*},
    TyModule, TypeId,
};

#[derive(Debug)]
pub struct TyProgram {
    pub kind: TyProgramKind,
    pub root: TyModule,
    pub storage_slots: Vec<StorageSlot>,
    pub logged_types: Vec<TypeId>,
}

#[derive(Clone, Debug)]
pub enum TyProgramKind {
    Contract {
        abi_entries: Vec<TyFunctionDeclaration>,
        declarations: Vec<TyDeclaration>,
    },
    Library {
        name: Ident,
    },
    Predicate {
        main_function: TyFunctionDeclaration,
        declarations: Vec<TyDeclaration>,
    },
    Script {
        main_function: TyFunctionDeclaration,
        declarations: Vec<TyDeclaration>,
    },
}

impl TyProgramKind {
    /// The parse tree type associated with this program kind.
    pub fn tree_type(&self) -> parsed::TreeType {
        match self {
            TyProgramKind::Contract { .. } => parsed::TreeType::Contract,
            TyProgramKind::Library { name } => parsed::TreeType::Library { name: name.clone() },
            TyProgramKind::Predicate { .. } => parsed::TreeType::Predicate,
            TyProgramKind::Script { .. } => parsed::TreeType::Script,
        }
    }
}
