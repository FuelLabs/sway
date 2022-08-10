mod utils;

use utils::{
    setup::get_contract_instance,
    wrappers::{clear, get, insert, is_empty, len, pop, push, remove, set, swap_remove},
};

// TODO: Replace many of the get calls with direct storage values
// once the SDK supports that

const TUP1: (u8, u8, u8) = (1, 1, 1);
const TUP2: (u8, u8, u8) = (2, 2, 2);
const TUP3: (u8, u8, u8) = (3, 3, 3);
const TUP4: (u8, u8, u8) = (4, 4, 4);
const TUP5: (u8, u8, u8) = (5, 5, 5);

mod success {
    use super::*;

    #[tokio::test]
    async fn can_get() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, TUP1).await;
        let item = get(&instance, 0).await;

        assert_eq!(item, TUP1);
    }

    #[tokio::test]
    async fn can_push() {
        let (instance, _id) = get_contract_instance().await;

        let len_before_push = len(&instance).await;
        assert_eq!(len_before_push, 0);

        push(&instance, TUP1).await;
        let item = get(&instance, 0).await;

        assert_eq!(item, TUP1);

        let len_after_push = len(&instance).await;
        assert_eq!(len_after_push, 1);
    }

    #[tokio::test]
    async fn can_pop() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, TUP1).await;
        let len_before_pop = len(&instance).await;
        assert_eq!(len_before_pop, 1);

        let item = pop(&instance).await;
        assert_eq!(item, TUP1);

        let len_after_pop = len(&instance).await;
        assert_eq!(len_after_pop, 0);
    }

    #[tokio::test]
    async fn can_remove() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, TUP1).await;
        push(&instance, TUP2).await;
        push(&instance, TUP3).await;
        push(&instance, TUP4).await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 4);

        assert_eq!(TUP1, get(&instance, 0).await);
        assert_eq!(TUP2, get(&instance, 1).await);
        assert_eq!(TUP3, get(&instance, 2).await);
        assert_eq!(TUP4, get(&instance, 3).await);

        let item = remove(&instance, 2).await;
        assert_eq!(item, TUP3);

        assert_eq!(TUP1, get(&instance, 0).await);
        assert_eq!(TUP2, get(&instance, 1).await);
        assert_eq!(TUP4, get(&instance, 2).await);

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 3);
    }

    #[tokio::test]
    async fn can_swap_remove() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, TUP1).await;
        push(&instance, TUP2).await;
        push(&instance, TUP3).await;
        push(&instance, TUP4).await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 4);

        assert_eq!(TUP1, get(&instance, 0).await);
        assert_eq!(TUP2, get(&instance, 1).await);
        assert_eq!(TUP3, get(&instance, 2).await);
        assert_eq!(TUP4, get(&instance, 3).await);

        let item = swap_remove(&instance, 1).await;
        assert_eq!(item, TUP2);

        assert_eq!(TUP1, get(&instance, 0).await);
        assert_eq!(TUP4, get(&instance, 1).await);
        assert_eq!(TUP3, get(&instance, 2).await);

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 3);
    }

    #[tokio::test]
    async fn can_insert() {
        let (instance, _id) = get_contract_instance().await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 0);

        insert(&instance, 0, TUP1).await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 1);

        push(&instance, TUP2).await;
        push(&instance, TUP3).await;
        push(&instance, TUP4).await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 4);

        assert_eq!(TUP1, get(&instance, 0).await);
        assert_eq!(TUP2, get(&instance, 1).await);
        assert_eq!(TUP3, get(&instance, 2).await);
        assert_eq!(TUP4, get(&instance, 3).await);

        insert(&instance, 1, TUP5).await;

        assert_eq!(TUP1, get(&instance, 0).await);
        assert_eq!(TUP5, get(&instance, 1).await);
        assert_eq!(TUP2, get(&instance, 2).await);
        assert_eq!(TUP3, get(&instance, 3).await);
        assert_eq!(TUP4, get(&instance, 4).await);

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 5);
    }

    #[tokio::test]
    async fn can_get_len() {
        let (instance, _id) = get_contract_instance().await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 0);

        push(&instance, TUP1).await;
        push(&instance, TUP2).await;
        push(&instance, TUP3).await;
        push(&instance, TUP4).await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 4);

        push(&instance, TUP4).await;
        let len_vec = len(&instance).await;

        assert_eq!(len_vec, 5);
    }

    #[tokio::test]
    async fn can_confirm_emptiness() {
        let (instance, _id) = get_contract_instance().await;

        let isempty = is_empty(&instance).await;
        assert_eq!(isempty, true);

        push(&instance, TUP1).await;
        push(&instance, TUP2).await;
        push(&instance, TUP3).await;
        push(&instance, TUP4).await;

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

        push(&instance, TUP1).await;
        push(&instance, TUP2).await;
        push(&instance, TUP3).await;
        push(&instance, TUP4).await;

        let isempty = is_empty(&instance).await;
        assert_eq!(isempty, false);

        clear(&instance).await;

        let isempty = is_empty(&instance).await;
        assert_eq!(isempty, true);
    }

    #[tokio::test]
    async fn can_set() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, TUP1).await;
        push(&instance, TUP2).await;
        push(&instance, TUP3).await;
        push(&instance, TUP4).await;

        assert_eq!(TUP1, get(&instance, 0).await);
        assert_eq!(TUP2, get(&instance, 1).await);
        assert_eq!(TUP3, get(&instance, 2).await);
        assert_eq!(TUP4, get(&instance, 3).await);

        set(&instance, 0, TUP4).await;
        set(&instance, 1, TUP3).await;
        set(&instance, 2, TUP2).await;
        set(&instance, 3, TUP1).await;

        assert_eq!(TUP4, get(&instance, 0).await);
        assert_eq!(TUP3, get(&instance, 1).await);
        assert_eq!(TUP2, get(&instance, 2).await);
        assert_eq!(TUP1, get(&instance, 3).await);
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

        insert(&instance, 1, TUP5).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Revert(0)")]
    async fn cant_set() {
        let (instance, _id) = get_contract_instance().await;

        set(&instance, 1, TUP5).await;
    }
}
