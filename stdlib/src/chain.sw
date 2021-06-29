library chain;

pub struct CallRequest {
  contract_id: byte32,
  coins_to_forward: u64,
  coin_color: u64,
  gas_to_forward: u64,
  params: u64, // TODO vec of params
  returns: u64, // TODO vec of mutable references
}


pub struct CallResponse {

}
