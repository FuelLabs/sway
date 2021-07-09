library chain;

// see https://github.com/FuelLabs/sway/issues/98#issuecomment-870873350 for details on these types
pub struct Address {
  inner: byte32
}

impl Address {
  fn new(addr: byte32) -> Self {
    Address {
      inner: addr
    }
  }
}

pub struct ContractId {
  inner: byte32
}

impl ContractId {
  fn new(id: byte32) -> Self {
    ContractId {
      inner: id  
    }
  }
}

pub struct Color {
  inner: byte32
}

impl Color {
  fn new(color: byte32) -> Self {
    Color {
      inner: color 
    }
  }
}

/// Some compiler magic is performed on this function and it shows up in the standard library
/// as `contract_caller()`.
pub fn std__contract_caller(address: byte32, t_name: TraitName) -> ContractCaller {
  // implemented in the compiler itself
}
