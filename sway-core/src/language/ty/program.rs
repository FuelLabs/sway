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
