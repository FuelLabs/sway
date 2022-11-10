contract;

struct S {
}

enum E {
}

abi MyContract {
    fn get_struct(s: S) -> S;
    fn get_enum(e: E) -> E;
}

impl MyContract for Contract {
    fn get_struct(s: S) -> S {
        s
    }

    fn get_enum(e: E) -> E {
        e
    }
}
