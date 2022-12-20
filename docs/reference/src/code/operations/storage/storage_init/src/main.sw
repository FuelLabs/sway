contract;

// ANCHOR: data_structures
struct Owner {
    maximum_owners: u64,
    role: Role,
}

impl Owner {
    fn default() -> Self {
        Self {
            maximum_owners: 10,
            role: Role::FullAccess,
        }
    }
}

enum Role {
    FullAccess: (),
    PartialAcess: (),
    NoAccess: (),
}
// ANCHOR_END: data_structures
// ANCHOR: initialization
storage {
    current_owners: u64 = 0,
    explicit_declaration: Owner = Owner {
        maximum_owners: 10,
        role: Role::FullAccess,
    },
    encapsulated_declaration: Owner = ~Owner::default(),
}
// ANCHOR_END: initialization
