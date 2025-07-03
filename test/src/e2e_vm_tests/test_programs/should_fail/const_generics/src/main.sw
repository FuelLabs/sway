// ignore garbage_collection_all_language_tests - needs a experimental feature
script;

struct S<const CRAZYN: u64> {}

fn main() {
    let _: S<UNKNOWN> = S { };
    let _: [u8; UNKNOWN] = [1u8];
    let _: str[UNKNOWN] = __to_str_array("abc");
}
