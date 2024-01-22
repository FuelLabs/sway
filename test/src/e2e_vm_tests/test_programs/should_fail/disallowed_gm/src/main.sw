script;

fn main() -> bool {
  // GM should be disallowed
  let is_caller_external = asm(r1) {
    gm r1 i1;
    r1: bool
  };
  is_caller_external
}
