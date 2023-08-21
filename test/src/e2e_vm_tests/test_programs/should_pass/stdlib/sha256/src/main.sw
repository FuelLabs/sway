script;

use std::hash::*;


fn sha256_str<T>(s: T) -> b256 {
    let mut hasher = Hasher::new();
    hasher.write_str(s);
    hasher.sha256()
}


fn main() -> bool {
  let test_sha256: b256 = 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08;
  assert(sha256_str("test") == test_sha256);

  let str_sha256: b256 = 0x4a3cd7c8b44dbf7941e55179425f746adeaa97fe2d99b571fffee78e9b41743c;
  assert(sha256_str("Fastest Modular Execution Layer!") == str_sha256);

  true
}

#[test]
fn test_works() {
    main();
}