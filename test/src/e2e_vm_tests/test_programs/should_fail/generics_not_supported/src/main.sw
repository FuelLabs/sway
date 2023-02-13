script;

dep lib_a;

fn main() -> u64 {

    let b = lib_a::inner_lib::<u64>::func();


    let c = lib_a::inner_lib::C::<u32>;


    let c = lib_a::inner_lib::<u32>::C;


    let s = lib_a::inner_lib::<u32>::S2 {};

    5
}
