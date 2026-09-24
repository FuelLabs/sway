script;

struct Wrapped {
    v: bool,
}

// These types decode fns should coalesce into
// only one in the final IR
configurable {
    WRAPPED: Wrapped = Wrapped { v: true },
    TUPLE: (bool,) = (false,),
}

fn main() -> bool {
    WRAPPED.v && TUPLE.0
}
