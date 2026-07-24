library;

pub fn check_args() {
    let _ = __mem_repr_id_runtime();
    let _ = __mem_repr_id_runtime::<u64, u32>();
    let _ = __mem_repr_id_runtime::<u64>(42u64);

    let _ = __mem_repr_id_encoding();
    let _ = __mem_repr_id_encoding::<u64, u32>();
    let _ = __mem_repr_id_encoding::<u64>(42u64);

    let _ = __mem_repr_id_hashing();
    let _ = __mem_repr_id_hashing::<u64, u32>();
    let _ = __mem_repr_id_hashing::<u64>(42u64);
}
