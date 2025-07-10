script;

enum Either<Left, Right> {
  Left: Left,
  Right: Right,
}

fn main() -> u64 {
   let thing: Either<bool, u64> = Either::Left::<bool, u64>(true);

   if let Either::Right(n) = thing {
       n
   } else {
       0
   }
}

// check: local u64 n
// check: local { u64, ( bool | u64 ) } thing

// check: get_local __ptr { u64, ( bool | u64 ) }, thing
// check: $(thing_var=$VAL) = get_local __ptr { u64, ( bool | u64 ) }, thing

// check: get_local __ptr { u64, ( bool | u64 ) }, __matched_value_1
// check: $(match_var=$VAL) = get_local __ptr { u64, ( bool | u64 ) }, __matched_value_1

// check: $(idx_val=$VAL) = const u64 0
// check: $(tag_ptr=$VAL) = get_elem_ptr $match_var, __ptr u64, $idx_val
// check: $(match_tag=$VAL) = load $tag_ptr

// check: $(one=$VAL) = const u64 1
// check: $(tags_match=$VAL) = call $(eq_fn=$ID)($match_tag, $one)
// check: cbr $tags_match, $(block0=$ID)(), $(block1=$ID)()

// check: $block0():
// check: $(match_var=$VAL) = get_local __ptr { u64, ( bool | u64 ) }, __matched_value_1

// check: $(idx_1_a=$VAL) = const u64 1
// check: $(idx_1_b=$VAL) = const u64 1
// check: $(variant_ptr=$VAL) = get_elem_ptr $match_var, __ptr u64, $idx_1_a, $idx_1_b
// check: $(thing_variant_val=$VAL) = load $variant_ptr

// check: $(n_var=$VAL) = get_local __ptr u64, n
// check: store $thing_variant_val to $n_var

// check: $(n_var=$VAL) = get_local __ptr u64, n
// check: $(n_val=$VAL) = load $n_var
// check: br $(block2=$ID)($n_val)

// check: $block1():
// check: $(zero=$VAL) = const u64 0
// check: br $block2($zero)

// check: $block2($(res=$VAL): u64):
// check: ret u64 $res

// check: fn $eq_fn(self $MD: u64, other $MD: u64) -> bool
