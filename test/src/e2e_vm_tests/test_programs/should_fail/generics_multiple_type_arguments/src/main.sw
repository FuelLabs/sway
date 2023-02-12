script;

dep lib_a;

fn main() -> u64 {

    let e = lib_a::inner_lib::MyEnum::<u64>::VariantA::<u64>;

    5
}
