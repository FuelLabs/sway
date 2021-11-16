script;

fn gimme_a_unit() {
    let x: () = ();
    x
}

fn also_gimme_a_unit() -> () {
    let x: () = ();
    x
}

fn main() -> u32 {
    gimme_a_unit();
    also_gimme_a_unit();
    123
}
