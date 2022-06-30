mod utils;

use utils::{
    setup::get_contract_instance,
    wrappers::*,
};


mod success {
    use super::*;

    #[tokio::test]
    async fn can_get_contract_id() { 
        let (_instance, _id) = get_contract_instance().await;
    }

    #[tokio::test]
    async fn can_push() { 
        let (instance, _id) = get_contract_instance().await;
    
        push(&instance, 50).await;
        let item = get(&instance, 0).await;
    
        assert_eq!(item, 50);
    }
    
    #[tokio::test]
    async fn can_pop() { 
        let (instance, _id) = get_contract_instance().await;
    
        push(&instance, 50).await;
        let item = pop(&instance).await;
    
        assert_eq!(item, 50);
    }
}