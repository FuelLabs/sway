script;

enum Fruit {
    Apple: (),
    Banana: (),
    Grapes: u64,
}

fn main() {
    let lunch = Fruit::Banana;
    eat(lunch);
    eat(Fruit::Grapes(3));
}

fn eat(meal: Fruit) -> bool {
    false
}

// regex: MD=!\d+
// regex: ANON=anon_\d+

// check: local ptr { u64, ( () | () | u64 ) } lunch

// check: const { u64, ( () | () | u64 ) } { u64 undef, ( () | () | u64 ) undef }

// check: fn $ANON(meal $MD: { u64, ( () | () | u64 ) }) -> bool
