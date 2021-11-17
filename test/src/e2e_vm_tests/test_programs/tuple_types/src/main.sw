script;

fn gimme_a_unit() {
    let x: () = ();
    x
}

fn also_gimme_a_unit() -> () {
    let x: () = ();
    x
}

fn gimme_a_pair() -> (u32, u64) {
    (1u32, 2u64)
}

fn main() -> u32 {
    gimme_a_unit();
    also_gimme_a_unit();
    let b = gimme_a_pair();
    123
}
