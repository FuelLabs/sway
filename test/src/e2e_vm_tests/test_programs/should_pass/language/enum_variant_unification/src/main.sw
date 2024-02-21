script;

struct S<T> {
    x: T,
}

fn main() -> u64 {
  // https://github.com/FuelLabs/sway/issues/5492
  let _ = foo();
  let _ = bar(true);

  // https://github.com/FuelLabs/sway/issues/5581
  let _: S<Option<u8>>  = S { x: Option::Some(1) };

  0
}

#[inline(never)]
fn foo() -> Option<u8> {
  match Some(true) {
    Option::Some(_b) => Option::Some(17),
    Option::None => Option::None,
  }
}

#[inline(never)]
fn bar(b: bool) -> Option<u8> {
   if(b) {
     Option::Some(19)
   } else {
     Option::None
   }
}