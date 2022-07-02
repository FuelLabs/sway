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

    #[tokio::test]
    async fn can_remove() { 
        let (instance, _id) = get_contract_instance().await;
    
        push(&instance, 50).await;
        push(&instance, 100).await;
        push(&instance, 150).await;
        push(&instance, 200).await;
        let item = remove(&instance, 2).await;
    
        assert_eq!(item, 150);
    }

    #[tokio::test]
    async fn can_swap_remove() { 
        let (instance, _id) = get_contract_instance().await;
    
        push(&instance, 50).await;
        push(&instance, 100).await;
        push(&instance, 150).await;
        push(&instance, 200).await;
        let item = swap_remove(&instance, 1).await;
        let new_item_at_index = get(&instance, 1).await;
    
        assert_eq!(item, 100);
        assert_eq!(new_item_at_index, 200);
    }

    #[tokio::test]
    async fn can_insert() { 
        let (instance, _id) = get_contract_instance().await;
    
        push(&instance, 50).await;
        push(&instance, 100).await;
        push(&instance, 150).await;
        push(&instance, 200).await;

        insert(&instance, 1, 250).await;

        let new_item_at_index = get(&instance, 1).await;
    
        assert_eq!(new_item_at_index, 250);
        assert_eq!(100, get(&instance, 2).await);
        assert_eq!(150, get(&instance, 3).await);
        assert_eq!(200, get(&instance, 4).await);
    }

    #[tokio::test]
    async fn can_get_len() { 
        let (instance, _id) = get_contract_instance().await;
    
        push(&instance, 50).await;
        push(&instance, 100).await;
        push(&instance, 150).await;
        push(&instance, 200).await;

        let len_vec = len(&instance).await;

        assert_eq!(len_vec, 4);

        push(&instance, 200).await;
        let len_vec = len(&instance).await;

        assert_eq!(len_vec, 5);
    }

    #[tokio::test]
    async fn can_confirm_emptiness() { 
        let (instance, _id) = get_contract_instance().await;
    
        push(&instance, 50).await;
        push(&instance, 100).await;
        push(&instance, 150).await;
        push(&instance, 200).await;

        let isempty = is_empty(&instance).await;

        assert_eq!(isempty, false);

        clear(&instance).await;

        let isempty = is_empty(&instance).await;

        assert_eq!(isempty, true);
    }
}