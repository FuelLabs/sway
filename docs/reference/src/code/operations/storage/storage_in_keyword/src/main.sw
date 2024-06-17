contract;

// ANCHOR: data_structures
struct Owner {
    maximum_owners: u64,
    role: Role,
}

impl Owner {
    // a constructor that can be evaluated to a constant `Owner` during compilation
    fn default() -> Self {
        Self {
            maximum_owners: 10,
            role: Role::FullAccess,
        }
    }
}

enum Role {
    FullAccess: (),
    PartialAccess: (),
    NoAccess: (),
}

// based on sha256("storage.current_owners")
const HASH_KEY: b256 = 0x84f905e3f560d70fbfab9ffcd92198998ce6f936e3d45f8fcb16b00f6a6a8d7e;

// ANCHOR_END: data_structures
// ANCHOR: initialization
storage {
    // ANCHOR: in_keyword
    new_current_owners in HASH_KEY: u64 = 0,
    // ANCHOR_END: in_keyword
    explicit_declaration: Owner = Owner {
        maximum_owners: 10,
        role: Role::FullAccess,
    },
    encapsulated_declaration: Owner = Owner::default(),
}
// ANCHOR_END: initialization
