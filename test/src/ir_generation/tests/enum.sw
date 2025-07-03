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

// check: $(temp_ptr=$VAL) = get_local __ptr { u64, ( () | () | u64 ) }, $(=__anon_\d+)
// check: $(idx_0=$VAL) = const u64 0
// check: $(tag_ptr=$VAL) = get_elem_ptr $temp_ptr, __ptr u64, $idx_0
// check: $(tag_val=$VAL) = const u64 1
// check: store $tag_val to $tag_ptr

// check: $(temp_val=$VAL) = load $temp_ptr
// check: $(lunch_ptr=$VAL) = get_local __ptr { u64, ( () | () | u64 ) }, lunch
// check: store $temp_val to $lunch_ptr

// check: $(lunch_ptr=$VAL) = get_local __ptr { u64, ( () | () | u64 ) }, lunch
// check: $(lunch_val=$VAL) = load $lunch_ptr
// check: call eat_0($lunch_val)

// check: $(temp_ptr=$VAL) = get_local __ptr { u64, ( () | () | u64 ) }, $(=__anon_\d+)
// check: $(idx_0=$VAL) = const u64 0
// check: $(tag_ptr=$VAL) = get_elem_ptr $temp_ptr, __ptr u64, $idx_0
// check: $(tag_val=$VAL) = const u64 2
// check: store $tag_val to $tag_ptr

// check: $(idx_1=$VAL) = const u64 1
// check: $(variant_val_ptr=$VAL) = get_elem_ptr $temp_ptr, __ptr u64, $idx_1
// check: $(num_grapes=$VAL) = const u64 3
// check: store $num_grapes to $variant_val_ptr

// check: $(temp_val=$VAL) = load $temp_ptr
// check: call eat_0($temp_val)
