script;

fn main(foo: u64) -> u64 {
    let ptr = __gtf::<raw_ptr>(0, 0xA); // SCRIPT_DATA
    let len = __gtf::<u64>(0, 0x4); // SCRIPT_DATA_LEN

    __log(ptr);
    __log(len);
    __log(foo);
    foo * 2
}
