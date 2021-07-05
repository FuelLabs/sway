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
      inner: addr
    }
  }
}

pub struct Color {
  inner: byte32
}

impl Color {
  fn new(color: byte32) -> Self {
    Color {
      inner: addr
    }
  }
}

pub struct CallRequest {
  contract_id: ContractId,
  coins_to_forward: u64,
  coin_color: Color,
  gas_to_forward: u64,
  params: u64, // TODO vec of params
  returns: u64, // TODO vec of mutable references
}


pub struct CallResponse {

}
