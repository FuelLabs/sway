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

const HASH_KEY: b256 = 0x7616e5793ef977b22465f0c843bcad56155c4369245f347bcc8a61edb08b7645;

// ANCHOR_END: data_structures
// ANCHOR: initialization
storage {
    // ANCHOR: in_keyword
    current_owners in HASH_KEY: u64 = 0,
    // ANCHOR_END: in_keyword
    explicit_declaration: Owner = Owner {
        maximum_owners: 10,
        role: Role::FullAccess,
    },
    encapsulated_declaration: Owner = Owner::default(),
}
// ANCHOR_END: initialization
