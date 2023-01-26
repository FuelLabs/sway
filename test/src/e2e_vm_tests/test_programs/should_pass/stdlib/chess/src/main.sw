script;

dep huge_enum;

use huge_enum::*;

fn main() -> u64 {
    let huge = Huge::b3;
    huge.to_u64()
}
