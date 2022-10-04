script;

use auth_testing_abi::AuthTesting;
use std::contract_id::ContractId;
use std::chain::auth::*;
use std::result::*;

fn main() -> u64 {
    // TODO: ContractId for auth_testing_contract should ideally be passed to script as an arg when possible.
    let auth_contract = abi(AuthTesting, 0x377fd69456e97da7456331c18a859c9eb3ce741268c299eaea0167c0eff678ad);
    let auth_caller_contract = ~ContractId::from(0x2fc63a758319acb31e34cbc2853b5ae4068b81dacb674db15b0b6d8d7dac074a);
    let value = auth_contract.returns_msg_sender(auth_caller_contract);
    if !value { 0 } else { 1 }
}
