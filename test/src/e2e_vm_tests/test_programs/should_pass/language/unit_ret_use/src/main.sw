script;

#[inline(never)]
fn a(x: u64) -> () {
   log(x);
}

#[inline(never)]
fn b(x: ()) -> () {

}

fn main() -> u64 {
   let x = a(1);
   b(x);
   2
}