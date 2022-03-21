script;

fn main() {
  // GM should be disallowed
  asm(r1) {
    gm r1 i1;
  };
}
