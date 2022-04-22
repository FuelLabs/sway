script;

fn main() {
  // this asm block should return unit, i.e. nothing
  asm(r1: 5) {
    r1: u64
  }
}
