mod utils;

use fuels::{prelude::*, tx::ContractId};

use utils::get_contract_instance;

#[tokio::test]
async fn can_get_contract_id() { 
    let (_instance, _id) = get_contract_instance().await;

    // Now you have an instance of your contract you can use to test each function
}