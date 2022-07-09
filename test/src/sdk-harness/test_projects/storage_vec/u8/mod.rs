mod utils;

use utils::{
    setup::get_contract_instance,
    wrappers::{clear, get, insert, is_empty, len, pop, push, remove, swap_remove},
};

// TODO: Replace many of the get calls with direct storage values
// once the SDK supports that

mod success {
    use super::*;

    #[tokio::test]
    async fn can_get() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, 50).await;
        let item = get(&instance, 0).await;

        assert_eq!(item, 50);
    }

    #[tokio::test]
    async fn can_push() {
        let (instance, _id) = get_contract_instance().await;

        let len_before_push = len(&instance).await;
        assert_eq!(len_before_push, 0);

        push(&instance, 50).await;
        let item = get(&instance, 0).await;

        assert_eq!(item, 50);

        let len_after_push = len(&instance).await;
        assert_eq!(len_after_push, 1);
    }

    #[tokio::test]
    async fn can_pop() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, 50).await;
        let len_before_pop = len(&instance).await;
        assert_eq!(len_before_pop, 1);

        let item = pop(&instance).await;
        assert_eq!(item, 50);

        let len_after_pop = len(&instance).await;
        assert_eq!(len_after_pop, 0);
    }

    #[tokio::test]
    async fn can_remove() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, 50).await;
        push(&instance, 100).await;
        push(&instance, 150).await;
        push(&instance, 200).await;

        let old_item_at_index = get(&instance, 2).await;
        assert_eq!(150, old_item_at_index);

        let item = remove(&instance, 2).await;

        assert_eq!(item, 150);

        let new_item_at_index = get(&instance, 2).await;
        assert_eq!(new_item_at_index, 200);
    }

    #[tokio::test]
    async fn can_swap_remove() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, 50).await;
        push(&instance, 100).await;
        push(&instance, 150).await;
        push(&instance, 200).await;

        let old_item_at_index = get(&instance, 1).await;
        assert_eq!(old_item_at_index, 100);

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

        let old_item_at_index = get(&instance, 1).await;
        assert_eq!(100, old_item_at_index);

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

        let len_vec = len(&instance).await;

        assert_eq!(len_vec, 0);

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

        let isempty = is_empty(&instance).await;
        assert_eq!(isempty, true);

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

    #[tokio::test]
    async fn can_clear() {
        let (instance, _id) = get_contract_instance().await;

        let isempty = is_empty(&instance).await;
        assert_eq!(isempty, true);

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

// Some of these are meant to be tests for None returns but since the SDK doesnt support options;
// the options are being unwrapped in the contract and tested as reverts instead
mod failure {
    use super::*;

    #[tokio::test]
    #[should_panic(expected = "Revert(0)")]
    async fn cant_get() {
        let (instance, _id) = get_contract_instance().await;

        get(&instance, 0).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(0)")]
    async fn cant_pop() {
        let (instance, _id) = get_contract_instance().await;

        pop(&instance).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(0)")]
    async fn cant_remove() {
        let (instance, _id) = get_contract_instance().await;

        let _ = remove(&instance, 0).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(0)")]
    async fn cant_swap_remove() {
        let (instance, _id) = get_contract_instance().await;

        let _ = swap_remove(&instance, 0).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(0)")]
    async fn cant_insert() {
        let (instance, _id) = get_contract_instance().await;

        insert(&instance, 1, 250).await;
    }
}
