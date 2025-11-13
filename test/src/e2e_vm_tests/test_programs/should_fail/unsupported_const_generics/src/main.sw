// ignore garbage_collection_all_language_tests - needs a experimental feature
script;

trait A {}

struct RepeatedConstGenericsNameInStructs<const N: u64, const N: u64> { }
enum RepeatedConstGenericsNameInEnums<const N: u64, const N: u64> { }
trait RepeatedConstGenericsNameInTraits<const N: u64, const N: u64> { }

fn repeated_const_generics_name<const N: u64, const N: u64>() {
}

struct CrazyStruct<const N: u64> {}

impl<const N: u64, const N: u64> CrazyStruct<N> {
}

impl<const N: u64> CrazyStruct<N> {
    fn repeated_const_generics_name<const N: u64, const A: u64, const A: u64>() {
    }
}

impl<const N: u64> A for CrazyStruct<N> {
    fn repeated_const_generics_name_2<const N: u64, const A: u64, const A: u64>() {
    }
}

enum CrazyEnum<const N: u64> {
    A: ()
}

impl<const N: u64, const N: u64> CrazyEnum<N> {
}

impl<const N: u64> CrazyEnum<N> {
    fn repeated_const_generics_name<const N: u64, const A: u64, const A: u64>() {
    }
}

impl<const N: u64> A for CrazyEnum<N> {
    fn repeated_const_generics_name_2<const N: u64, const A: u64, const A: u64>() {
    }
}

// abi cannot have const generics

abi NoConstGenericsOnArgs {
    fn f(a: CrazyStruct<1>);
}

abi NoConstGenericsOnReturn {
    fn f() -> CrazyEnum<1>;
}

struct StructWithConstGenericInside {
    a: CrazyStruct<1>,
}

abi NoConstGenericsIndirectStruct {
    fn f() -> StructWithConstGenericInside;
}

enum EnumWithConstGenericInside {
    A: CrazyStruct<1>,
}

abi NoConstGenericsIndirectEnum {
    fn f() -> EnumWithConstGenericInside;
}

// configurable cannot have const generics

configurable {
    A: CrazyStruct<1> = CrazyStruct { },
}

fn main() {
    let _: CrazyStruct<UNKNOWN> = CrazyStruct { };
    let _: CrazyEnum<UNKNOWN> = CrazyEnum::A;
    let _: [u8; UNKNOWN] = [1u8];
    let _: str[UNKNOWN] = __to_str_array("abc");
}
