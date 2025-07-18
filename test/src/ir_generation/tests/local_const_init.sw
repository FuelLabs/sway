script;

struct S {
  s : u64
}

fn s(x : u64) -> S {
  S { s: x }
}

fn main() -> u64 {
  // unsigned integers
  const A = !0u8;
  const B = !0u16;
  const C = !0u32;
  const D = !0u64;

  // bool
  const E = !true;

  const X = s(1);
  X.s
}

// check:        local u8 A
// check:        local u64 B
// check:        local u64 C
// check:        local u64 D
// check:        local bool E
// check:        local { u64 } X

// check: $(a_var=$VAL) = const u8 255
// check: $(b_var=$VAL) = const u64 65535
// check: $(c_var=$VAL) = const u64 4294967295
// check: $(d_var=$VAL) = const u64 18446744073709551615
// check: $(e_var=$VAL) = const bool false

// check: $(x_var=$VAL) = get_local __ptr { u64 }, X
// check: $(one=$VAL) = const { u64 } { u64 1 }
// not: call
// check: store $one to $x_var
