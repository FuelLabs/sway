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

// check: get_local { u64, ( bool | u64 ) } thing

// check: $(thing_var=$VAL) = get_local { u64, ( bool | u64 ) } thing
// check: $(thing_tag=$VAL) = extract_value $thing_var, { u64, ( bool | u64 ) }, 0
// check: $(one=$VAL) = const u64 1
// check: $(tags_match=$VAL) = call $(eq_fn=$ID)($thing_tag, $one)
// check: cbr $tags_match, $(block0=$ID)(), $(block1=$ID)()

// check: $block0():
// check: $(thing_var=$VAL) = get_local { u64, ( bool | u64 ) } thing
// check: $(thing_variant_val=$VAL) = extract_value $thing_var, { u64, ( bool | u64 ) }, 1, 1
// check: $(n_var=$VAL) = get_local u64 n
// check: store $thing_variant_val to $n_var

// check: $(n_var=$VAL) = get_local u64 n
// check: $(n_val=$VAL) = load $n_var
// check: br $(block2=$ID)($n_val)

// check: $block1():
// check: $(zero=$VAL) = const u64 0
// check: br $block2($zero)

// check: $block2($(res=$VAL): u64):
// check: ret u64 $res

// check: fn $eq_fn(self $MD: u64, other $MD: u64) -> bool
