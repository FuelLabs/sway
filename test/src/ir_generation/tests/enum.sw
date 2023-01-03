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

// check: local { u64, ( () | () | u64 ) } lunch

// check: $(enum_undef=$VAL) = get_local { u64, ( () | () | u64 ) } $ID
// check: $(one_val=$VAL) = const u64 1
// check: $(enum_tagged=$VAL) = insert_value $enum_undef, { u64, ( () | () | u64 ) }, $one_val, 0
// check: $(lunch_var=$VAL) = get_local { u64, ( () | () | u64 ) } lunch
// check: store $enum_tagged to $lunch_var

// check: $(lunch_var=$VAL) = get_local { u64, ( () | () | u64 ) } lunch
// check: call $(eat_fn=$ID)($lunch_var)

// check: $(enum_undef=$VAL) = get_local { u64, ( () | () | u64 ) } $ID
// check: $(two_val=$VAL) = const u64 2
// check: $(enum_tagged=$VAL) = insert_value $enum_undef, { u64, ( () | () | u64 ) }, $two_val, 0
// check: $(three_val=$VAL) = const u64 3
// check: $(enum_init=$VAL) = insert_value $enum_tagged, { u64, ( () | () | u64 ) }, $three_val, 1
// check: call $ID($enum_init)

// check: fn $eat_fn(meal $MD: { u64, ( () | () | u64 ) }) -> bool
