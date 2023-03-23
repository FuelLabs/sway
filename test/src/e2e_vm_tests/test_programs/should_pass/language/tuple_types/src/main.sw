script;

fn gimme_a_unit() {
    let x: () = ();
    x
}

fn also_gimme_a_unit() -> () {
    let x: () = ();
    x
}

fn gimme_a_single_value() -> (u32,) {
    let x: (u32,) = (123u32,);
    x
}

fn gimme_a_pair() -> (u32, u64) {
    (1u32, 2u64)
}

fn main() -> u32 {
    gimme_a_unit();
    also_gimme_a_unit();
    let _x = gimme_a_single_value();
    let _b = gimme_a_pair();
    123
}
