script;

struct Wrapped {
    v: u64,
}

// These types decode fns should coalesce into 
// only one in the final IR
configurable {
    WRAPPED: Wrapped = Wrapped { v: 1 },
    TUPLE: (u64,) = (2,),
}

fn main() -> u64 {
    WRAPPED.v + TUPLE.0
}
