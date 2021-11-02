library storage;
// These methods will all be replaced by generic functions when those come in. 
// See https://github.com/FuelLabs/sway/issues/272 for details.


pub fn store_u64(key: b256, value: u64) {
  asm(r1: key, r2: value) {
    sww r1 r2;
  };
}

pub fn get_u64(key: b256) -> u64 {
  asm(r1: key, r2) {
    srw r2 r1;
    r2: u64
  }
}

pub fn store_u32(key: b256, value: u32) {
  asm(r1: key, r2: value) {
    sww r1 r2;
  };
}

pub fn get_u32(key: b256) -> u32 {
  asm(r1: key, r2) {
    srw r2 r1;
    r2: u32
  }
}

pub fn store_u16(key: b256, value: u16) {
  asm(r1: key, r2: value) {
    sww r1 r2;
  };
}

pub fn get_u16(key: b256) -> u16 {
  asm(r1: key, r2) {
    srw r2 r1;
    r2: u16
  }
}

pub fn store_u8(key: b256, value: u8) {
  asm(r1: key, r2: value) {
    sww r1 r2;
  };
}

pub fn get_u8(key: b256) -> u8 {
  asm(r1: key, r2) {
    srw r2 r1;
    r2: u8
  }
}

pub fn store_bool(key: b256, value: bool) {
  asm(r1: key, r2: value) {
    sww r1 r2;
  };
}

pub fn get_bool(key: b256) -> bool {
  asm(r1: key, r2) {
    srw r2 r1;
    r2: bool
  }
}

pub fn store_byte(key: b256, value: byte) {
  asm(r1: key, r2: value) {
    sww r1 r2;
  };
}

pub fn get_byte(key: b256) -> byte {
  asm(r1: key, r2) {
    srw r2 r1;
    r2: byte
  }
}
