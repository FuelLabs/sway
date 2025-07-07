// ignore garbage_collection_all_language_tests - needs a experimental feature
script;

struct RepeatedConstGenericsNameInStructs<const N: u64, const N: u64> { }

impl<const N: u64, const N: u64> RepeatedConstGenericsNameInStructs<N, N> {
    fn repeated_const_generics_name<const N: u64, const A: u64, const A: u64>() {
    }
}

enum RepeatedConstGenericsNameInEnums<const N: u64, const N: u64> { }

fn repeated_const_generics_name<const N: u64, const N: u64>() {
}

struct CrazyStruct<const N: u64> {}
enum CrazyEnum<const N: u64> {
    A: ()
}

fn main() {
    let _: CrazyStruct<UNKNOWN> = CrazyStruct { };
    let _: CrazyEnum<UNKNOWN> = CrazyEnum::A;
    let _: [u8; UNKNOWN] = [1u8];
    let _: str[UNKNOWN] = __to_str_array("abc");
}
