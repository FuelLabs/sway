mod utils;

use utils::{
    setup::get_contract_instance,
    wrappers::{clear, get, insert, is_empty, len, pop, push, remove, swap_remove},
};

// TODO: Replace many of the get calls with direct storage values
// once the SDK supports that

const BYTES1: [u8; 32] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31];
const BYTES2: [u8; 32] = [32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63];
const BYTES3: [u8; 32] = [64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95];
const BYTES4: [u8; 32] = [96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127];
const BYTES5: [u8; 32] = [128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159];

mod success {
    use super::*;

    #[tokio::test]
    async fn can_get() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, BYTES1).await;
        let item = get(&instance, 0).await;

        assert_eq!(item, BYTES1);
    }

    #[tokio::test]
    async fn can_push() {
        let (instance, _id) = get_contract_instance().await;

        let len_before_push = len(&instance).await;
        assert_eq!(len_before_push, 0);

        push(&instance, BYTES1).await;
        let item = get(&instance, 0).await;

        assert_eq!(item, BYTES1);

        let len_after_push = len(&instance).await;
        assert_eq!(len_after_push, 1);
    }

    #[tokio::test]
    async fn can_pop() {
        let (instance, _id) = get_contract_instance().await;

        push(&instance, BYTES1).await;
        let len_before_pop = len(&instance).await;
        assert_eq!(len_before_pop, 1);

        let item = pop(&instance).await;
        assert_eq!(item, BYTES1);

        let len_after_pop = len(&instance).await;
        assert_eq!(len_after_pop, 0);
    }

    #[tokio::test]
    async fn can_remove() {
        let (instance, _id) = get_contract_instance().await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 0);

        push(&instance, BYTES1).await;
        push(&instance, BYTES2).await;
        push(&instance, BYTES3).await;
        push(&instance, BYTES4).await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 4);

        assert_eq!(BYTES1, get(&instance, 0).await);
        assert_eq!(BYTES2, get(&instance, 1).await);
        assert_eq!(BYTES3, get(&instance, 2).await);
        assert_eq!(BYTES4, get(&instance, 3).await);

        let item = remove(&instance, 2).await;
        assert_eq!(item, BYTES3);

        assert_eq!(BYTES1, get(&instance, 0).await);
        assert_eq!(BYTES2, get(&instance, 1).await);
        assert_eq!(BYTES4, get(&instance, 2).await);

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 3);
    }

    #[tokio::test]
    async fn can_swap_remove() {
        let (instance, _id) = get_contract_instance().await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 0);

        push(&instance, BYTES1).await;
        push(&instance, BYTES2).await;
        push(&instance, BYTES3).await;
        push(&instance, BYTES4).await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 4);

        assert_eq!(BYTES1, get(&instance, 0).await);
        assert_eq!(BYTES2, get(&instance, 1).await);
        assert_eq!(BYTES3, get(&instance, 2).await);
        assert_eq!(BYTES4, get(&instance, 3).await);

        let item = swap_remove(&instance, 1).await;
        assert_eq!(item, BYTES2);

        assert_eq!(BYTES1, get(&instance, 0).await);
        assert_eq!(BYTES4, get(&instance, 1).await);
        assert_eq!(BYTES3, get(&instance, 2).await);

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 3);
    }

    #[tokio::test]
    async fn can_insert() {
        let (instance, _id) = get_contract_instance().await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 0);

        push(&instance, BYTES1).await;
        push(&instance, BYTES2).await;
        push(&instance, BYTES3).await;
        push(&instance, BYTES4).await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 4);

        assert_eq!(BYTES1, get(&instance, 0).await);
        assert_eq!(BYTES2, get(&instance, 1).await);
        assert_eq!(BYTES3, get(&instance, 2).await);
        assert_eq!(BYTES4, get(&instance, 3).await);

        insert(&instance, 1, BYTES5).await;

        assert_eq!(BYTES1, get(&instance, 0).await);
        assert_eq!(BYTES5, get(&instance, 1).await);
        assert_eq!(BYTES2, get(&instance, 2).await);
        assert_eq!(BYTES3, get(&instance, 3).await);
        assert_eq!(BYTES4, get(&instance, 4).await);

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 5);
    }

    #[tokio::test]
    async fn can_get_len() {
        let (instance, _id) = get_contract_instance().await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 0);

        push(&instance, BYTES1).await;
        push(&instance, BYTES2).await;
        push(&instance, BYTES3).await;
        push(&instance, BYTES4).await;

        let len_vec = len(&instance).await;
        assert_eq!(len_vec, 4);

        push(&instance, BYTES5).await;
        let len_vec = len(&instance).await;

        assert_eq!(len_vec, 5);
    }

    #[tokio::test]
    async fn can_confirm_emptiness() {
        let (instance, _id) = get_contract_instance().await;

        let isempty = is_empty(&instance).await;
        assert_eq!(isempty, true);

        push(&instance, BYTES1).await;
        push(&instance, BYTES2).await;
        push(&instance, BYTES3).await;
        push(&instance, BYTES4).await;

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

        push(&instance, BYTES1).await;
        push(&instance, BYTES2).await;
        push(&instance, BYTES3).await;
        push(&instance, BYTES4).await;

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

        insert(&instance, 1, BYTES5).await;
    }
}
