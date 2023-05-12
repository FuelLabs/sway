script;

mod lib_a;

fn main() -> u64 {

    let _b = lib_a::inner_lib::<u64>::func();


    let _c = lib_a::inner_lib::C::<u32>;


    let _c = lib_a::inner_lib::<u32>::C;


    let _s = lib_a::inner_lib::<u32>::S2 {};

    5
}
