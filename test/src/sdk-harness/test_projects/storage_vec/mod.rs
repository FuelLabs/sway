mod utils;

use utils::*;

#[tokio::test]
async fn can_get_contract_id() { 
    let (_instance, _id) = get_contract_instance().await;

    // Now you have an instance of your contract you can use to test each function
}

#[tokio::test]
async fn can_push() { 
    let (instance, _id) = get_contract_instance().await;

    push(&instance, 50).await;
    let item = get(&instance, 0).await;

    assert_eq!(item, 50);
}