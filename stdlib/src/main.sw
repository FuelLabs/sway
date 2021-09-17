library std;

dep chain;
dep ops;

/// Returns the caller of this contract. Reverts if there is no caller.
pub fn caller() -> b256 {
  // check $fp->saved_registers and if that has an $fp, check that until $fp->saved_registers is 0x00...
  // return the rover's 0th b256 as the contract ID
  asm(r1) {

  }
}
