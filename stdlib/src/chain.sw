library chain;

/*
/// Returns the caller of this contract. Reverts if there is no caller.
pub fn caller() -> b256 {
  // check $fp->saved_registers and if that has an $fp, check that until $fp->saved_registers is 0x00...
  // return the rover's 0th b256 as the contract ID
}
*/

// the saved registers for the last caller start at byte 64 and go until byte 736, or word 8 until word 92. The `to` value is byte 0 of the frame pointer.
pub fn saved_to_value() -> b256 {
  asm() {
    fp: b256
  }
}
