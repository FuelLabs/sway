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

// check: local ptr u64 n
// check: local ptr { u64, ( bool | u64 ) } thing

// check: get_ptr ptr { u64, ( bool | u64 ) } thing, ptr { u64, ( bool | u64 ) }, 0

// check: $(thing_ptr=$VAL) = get_ptr ptr { u64, ( bool | u64 ) } thing, ptr { u64, ( bool | u64 ) }, 0
// check: $(thing_tag=$VAL) = extract_value $thing_ptr, { u64, ( bool | u64 ) }, 0
// check: $(one=$VAL) = const u64 1
// check: $(tags_match=$VAL) = call $(eq_fn=$ID)($thing_tag, $one)
// check: cbr $tags_match, $(block0=$ID)(), $(block1=$ID)()

// check: $block0():
// check: $(thing_ptr=$VAL) = get_ptr ptr { u64, ( bool | u64 ) } thing, ptr { u64, ( bool | u64 ) }, 0
// check: $(thing_variant_val=$VAL) = extract_value $thing_ptr, { u64, ( bool | u64 ) }, 1, 1
// check: $(n_ptr=$VAL) = get_ptr ptr u64 n, ptr u64, 0
// check: store $thing_variant_val, ptr $n_ptr

// check: $(n_ptr=$VAL) = get_ptr ptr u64 n, ptr u64, 0
// check: $(n_val=$VAL) = load ptr $n_ptr
// check: br $(block2=$ID)($n_val)

// check: $block1():
// check: $(zero=$VAL) = const u64 0
// check: br $block2($zero)

// check: $block2($(res=$VAL): u64):
// check: ret u64 $res

// check: fn $eq_fn(self $MD: u64, other $MD: u64) -> bool
