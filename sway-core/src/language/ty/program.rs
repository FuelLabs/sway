use fuel_tx::StorageSlot;
use sway_types::Ident;

use crate::{language::parsed::TreeType, TyDeclaration, TyFunctionDeclaration, TyModule, TypeId};

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
    pub fn tree_type(&self) -> TreeType {
        match self {
            TyProgramKind::Contract { .. } => TreeType::Contract,
            TyProgramKind::Library { name } => TreeType::Library { name: name.clone() },
            TyProgramKind::Predicate { .. } => TreeType::Predicate,
            TyProgramKind::Script { .. } => TreeType::Script,
        }
    }
}
