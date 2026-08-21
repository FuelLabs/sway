library;

pub fn check_args() {
    let _ = __mem_repr_eq::<u64>();
    let _ = __mem_repr_eq::<u64>("runtime");
    let _ = __mem_repr_eq::<u64>("runtime", "encoding", "hashing");

    let _ = __mem_repr_eq("runtime", "encoding");
    let _ = __mem_repr_eq::<u64, u32>("runtime", "encoding");

    let _ = __mem_repr_eq::<u64>(42u64, "encoding");

    let _ = __mem_repr_eq::<u64>("runtime", "invalid");

    let repr = non_constant_repr(); // TODO-DCA: Fix invalid DCA warning that `repr` is not used.
    __mem_repr_eq::<u64>(repr, "encoding");
}

fn non_constant_repr() -> str {
    "runtime"
}
