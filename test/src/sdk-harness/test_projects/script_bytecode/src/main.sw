script;

use std::hash::*;
use std::tx::tx_script_bytecode;

fn main() -> b256 {
    // length of return array is script length padded to nearest full word
    let script_bytecode: [u64; 35] = tx_script_bytecode().unwrap();

    // Return the hash of the bytecode to compare
    sha256(script_bytecode)
}
