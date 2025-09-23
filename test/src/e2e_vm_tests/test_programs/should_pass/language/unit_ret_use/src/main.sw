library;

#[inline(never)]
fn a(x: u64) -> () {
   return_unit(x);
}

#[inline(never)]
fn b(_x: ()) -> () {}

fn return_unit(_x: u64) {}

pub fn main() -> u64 {
   let x = a(1);
   b(x);
   2
}

#[test]
fn test() {
   assert(main() == 2);
}
