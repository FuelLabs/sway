contract;

// Error: Indexed field on non event attributed type
struct MyStruct {
  #[indexed]
  a: u32
}

// Error: Indexed field is not a fixed size type
#[event]
struct MyStruct2 {
  #[indexed]
  v: Vec<u64>
}

// Error: Indexed field is not an initially sequential field
#[event]
struct MyStruct3 {
  a: u32,
  #[indexed]
  b: u32
}

// Error: Indexed fields must be sequential
#[event]
struct MyStruct4 {
  #[indexed]
  a: u8,
  b: [u64; 32],
  #[indexed]
  c: u64,
}
