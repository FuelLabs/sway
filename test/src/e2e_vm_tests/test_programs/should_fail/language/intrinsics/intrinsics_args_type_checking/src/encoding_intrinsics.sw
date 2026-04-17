library;

pub fn check_args() {
    let _ = __encode_buffer_append();
    let _ = __encode_buffer_append(42u64);
    let _ = __encode_buffer_append((), 42u64);
    let _ = __encode_buffer_append((__addr_of(0), 0u64, 0u64), (1u32, 1u64));

    let _ = __encode_buffer_empty(42u64);
    let _ = __encode_buffer_empty((), 42u64);
}
