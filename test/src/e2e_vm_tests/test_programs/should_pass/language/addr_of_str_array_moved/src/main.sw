script;

use std::string::String;

fn main() {
    let string_array = __to_str_array("ABCDEF");
    let string = String::from_moved_raw_slice(
        __transmute::<(raw_ptr, u64), raw_slice>((__addr_of(string_array), 6)),
    );
    poke(string.ptr());
    poke(__addr_of(string_array));
}

#[inline(never)]
fn poke<T>(_t: T) {}
