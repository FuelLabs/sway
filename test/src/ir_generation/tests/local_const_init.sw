script;

struct S {
  s : u64
}

fn s(x : u64) -> S {
  S { s: x }
}

fn return_49() -> u64 { 7 * 7 }
fn return_times_8(v: u64) -> u64 { v * 8 }
fn max(l: u64, r: u64) -> u64 {
  if l > r { l } else { r }
}

fn main() -> u64 {
  let A: u64 = 0 + 1 + 2 + 3 + 4 + 5;
  let B = return_49();
  let C = return_times_8(8);
  let D = max(23, 45);
  
  const X = s(1);
  X.s
}

// check:        local u64 A
// check:        local u64 B
// check:        local u64 C
// check:        local u64 D
// check:        local { u64 } X

// not: call
// check: $(A_var=$VAL) = get_local ptr u64, A
// check: const u64 15

// not: call
// check: $(B_var=$VAL) = get_local ptr u64, B
// check: const u64 49

// not: call
// check: $(C_var=$VAL) = get_local ptr u64, C
// check: const u64 64

// not: call
// check: $(D_var=$VAL) = get_local ptr u64, D
// check: const u64 45

// check: $(x_var=$VAL) = get_local ptr { u64 }, X
// check: $(one=$VAL) = const { u64 } { u64 1 }
// not: call
// check: store $one to $x_var
