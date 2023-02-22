#[macro_export]
macro_rules! testgen {
    (
        $module_name:ident,
        $abi_path:expr,
        $type_label:expr,
        $type_declaration:ty,
        $arg0:expr,
        $arg1:expr,
        $arg2:expr,
        $arg3:expr,
        $arg4:expr
    ) => {
        pub mod $module_name {
            use fuels::prelude::*;

            abigen!(Contract(
                name = "MyContract",
                abi = $abi_path,
            ));

            pub mod setup {
                use super::*;

                pub async fn get_contract_instance() -> MyContract {
                    let wallet = launch_provider_and_get_wallet().await;

                    let id = Contract::deploy(
                        &format!(
                            "test_artifacts/storage_vec/svec_{}/out/debug/svec_{}.bin",
                            $type_label,
                            $type_label,
                        ),
                        &wallet,
                        TxParameters::new(None, Some(100_000_000), None),
                        StorageConfiguration::with_storage_path(Some(
                            format!(
                                "test_artifacts/storage_vec/svec_{}/out/debug/svec_{}-storage_slots.json",
                                $type_label,
                                $type_label,
                            ),
                        )),
                    ).await.unwrap();

                    MyContract::new(id.clone(), wallet)
                }
            }

            pub mod wrappers {
                use super::*;

                // TODO: tx params
                pub async fn push(instance: &MyContract, value: $type_declaration) {
                    instance.methods()
                        .push(value)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn get(instance: &MyContract, index: u64) -> $type_declaration {
                    instance.methods()
                        .get(index)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn pop(instance: &MyContract) -> $type_declaration {
                    instance.methods()
                        .pop()
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn remove(instance: &MyContract, index: u64) -> $type_declaration {
                    instance.methods()
                        .remove(index)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn swap_remove(instance: &MyContract, index: u64) -> $type_declaration {
                    instance.methods()
                        .swap_remove(index)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn set(instance: &MyContract, index: u64, value: $type_declaration) {
                    instance.methods()
                        .set(index, value)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn insert(instance: &MyContract, index: u64, value: $type_declaration) {
                    instance.methods()
                        .insert(index, value)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn len(instance: &MyContract) -> u64 {
                    instance.methods()
                        .len()
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn is_empty(instance: &MyContract) -> bool {
                    instance.methods()
                        .is_empty()
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn clear(instance: &MyContract) {
                    instance.methods()
                        .clear()
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn swap(instance: &MyContract, index_0: u64, index_1: u64) {
                    instance.methods()
                        .swap(index_0, index_1)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn first(instance: &MyContract) -> $type_declaration {
                    instance.methods()
                        .first()
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn last(instance: &MyContract) -> $type_declaration {
                    instance.methods()
                        .last()
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn reverse(instance: &MyContract) {
                    instance.methods()
                        .reverse()
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn fill(instance: &MyContract, value: $type_declaration) {
                    instance.methods()
                        .fill(value)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn resize(instance: &MyContract, new_len: u64, value: $type_declaration) {
                    instance.methods()
                        .resize(new_len, value)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn append(instance: &MyContract) {
                    instance.methods()
                        .append()
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn push_other(instance: &MyContract, value: $type_declaration) {
                    instance.methods()
                        .push_other(value)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
                }
            }

            pub mod success {
                use super::{
                    *,
                    setup::get_contract_instance,
                    wrappers::*,
                };

                #[tokio::test]
                async fn can_get() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;

                    assert_eq!(get(&instance, 0).await, $arg0);
                }

                #[tokio::test]
                async fn can_push() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;

                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(len(&instance).await, 1);
                }

                #[tokio::test]
                async fn can_pop() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;

                    assert_eq!(len(&instance).await, 1);
                    assert_eq!(pop(&instance).await, $arg0);
                    assert_eq!(len(&instance).await, 0);
                }

                #[tokio::test]
                async fn can_remove() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    assert_eq!(remove(&instance, 2).await, $arg2);

                    assert_eq!(len(&instance).await, 3);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg3);
                }

                #[tokio::test]
                async fn can_swap_remove() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    assert_eq!(swap_remove(&instance, 1).await, $arg1);

                    assert_eq!(len(&instance).await, 3);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg3);
                    assert_eq!(get(&instance, 2).await, $arg2);
                }

                #[tokio::test]
                async fn can_insert() {
                    let instance = get_contract_instance().await;

                    insert(&instance, 0, $arg0).await;

                    assert_eq!(len(&instance).await, 1);
                    assert_eq!(get(&instance, 0).await, $arg0);

                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    insert(&instance, 1, $arg4).await;

                    assert_eq!(len(&instance).await, 5);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg4);
                    assert_eq!(get(&instance, 2).await, $arg1);
                    assert_eq!(get(&instance, 3).await, $arg2);
                    assert_eq!(get(&instance, 4).await, $arg3);
                }

                #[tokio::test]
                async fn can_get_len() {
                    let instance = get_contract_instance().await;

                    assert_eq!(len(&instance).await, 0);

                    push(&instance, $arg0).await;

                    assert_eq!(len(&instance).await, 1);

                    push(&instance, $arg1).await;

                    assert_eq!(len(&instance).await, 2);
                }

                #[tokio::test]
                async fn can_confirm_emptiness() {
                    let instance = get_contract_instance().await;

                    assert!(is_empty(&instance).await);

                    push(&instance, $arg0).await;

                    assert!(!is_empty(&instance).await);

                    clear(&instance).await;

                    assert!(is_empty(&instance).await);
                }

                #[tokio::test]
                async fn can_clear() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;

                    clear(&instance).await;

                    assert!(is_empty(&instance).await);

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    clear(&instance).await;

                    assert!(is_empty(&instance).await);
                }

                #[tokio::test]
                async fn can_set() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    set(&instance, 0, $arg3).await;
                    set(&instance, 1, $arg2).await;
                    set(&instance, 2, $arg1).await;
                    set(&instance, 3, $arg0).await;

                    assert_eq!(get(&instance, 0).await, $arg3);
                    assert_eq!(get(&instance, 1).await, $arg2);
                    assert_eq!(get(&instance, 2).await, $arg1);
                    assert_eq!(get(&instance, 3).await, $arg0);
                }

                #[tokio::test]
                async fn can_swap() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    swap(&instance, 0, 3).await;
                    swap(&instance, 1, 2).await;

                    assert_eq!(get(&instance, 0).await, $arg3);
                    assert_eq!(get(&instance, 1).await, $arg2);
                    assert_eq!(get(&instance, 2).await, $arg1);
                    assert_eq!(get(&instance, 3).await, $arg0);
                }

                #[tokio::test]
                async fn can_get_first() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;

                    assert_eq!(first(&instance).await, $arg0);
                }

                #[tokio::test]
                async fn can_get_last() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;

                    assert_eq!(last(&instance).await, $arg1);
                }

                #[tokio::test]
                async fn can_reverse() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    reverse(&instance).await;

                    assert_eq!(get(&instance, 0).await, $arg3);
                    assert_eq!(get(&instance, 1).await, $arg2);
                    assert_eq!(get(&instance, 2).await, $arg1);
                    assert_eq!(get(&instance, 3).await, $arg0);

                    reverse(&instance).await;

                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);
                }

                #[tokio::test]
                async fn can_fill() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    fill(&instance, $arg4).await;

                    assert_eq!(get(&instance, 0).await, $arg4);
                    assert_eq!(get(&instance, 1).await, $arg4);
                    assert_eq!(get(&instance, 2).await, $arg4);
                    assert_eq!(get(&instance, 3).await, $arg4);
                }

                #[tokio::test]
                async fn can_resize_up() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    resize(&instance, 6, $arg4).await;

                    assert_eq!(len(&instance).await, 6);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);
                    assert_eq!(get(&instance, 4).await, $arg4);
                    assert_eq!(get(&instance, 5).await, $arg4);
                }

                #[tokio::test]
                async fn can_resize_down() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    resize(&instance, 2, $arg4).await;

                    assert_eq!(len(&instance).await, 2);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                }

                #[tokio::test]
                async fn can_append() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;
                    push_other(&instance, $arg0).await;
                    push_other(&instance, $arg1).await;
                    push_other(&instance, $arg2).await;
                    push_other(&instance, $arg3).await;

                    append(&instance).await;

                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);
                    assert_eq!(get(&instance, 4).await, $arg0);
                    assert_eq!(get(&instance, 5).await, $arg1);
                    assert_eq!(get(&instance, 6).await, $arg2);
                    assert_eq!(get(&instance, 7).await, $arg3);
                }
            }

            pub mod failure {
                use super::{
                    *,
                    setup::get_contract_instance,
                    wrappers::*,
                };

                #[tokio::test]
                #[should_panic(expected = "Revert(0)")]
                async fn cant_get() {
                    let instance = get_contract_instance().await;

                    get(&instance, 0).await;
                }

                #[tokio::test]
                #[should_panic(expected = "Revert(0)")]
                async fn cant_pop() {
                    let instance = get_contract_instance().await;

                    let _ = pop(&instance).await;
                }

                #[tokio::test]
                #[should_panic(expected = "Revert(18446744073709486084)")]
                async fn cant_remove() {
                    let instance = get_contract_instance().await;

                    let _ = remove(&instance, 0).await;
                }

                #[tokio::test]
                #[should_panic(expected = "Revert(18446744073709486084)")]
                async fn cant_swap() {
                    let instance = get_contract_instance().await;

                    let _ = swap(&instance, 0, 1).await;
                }

                #[tokio::test]
                #[should_panic(expected = "Revert(18446744073709486084)")]
                async fn cant_swap_remove() {
                    let instance = get_contract_instance().await;

                    let _ = swap_remove(&instance, 0).await;
                }

                #[tokio::test]
                #[should_panic(expected = "Revert(18446744073709486084)")]
                async fn cant_insert() {
                    let instance = get_contract_instance().await;

                    insert(&instance, 1, $arg1).await;
                }

                #[tokio::test]
                #[should_panic(expected = "Revert(18446744073709486084)")]
                async fn cant_set() {
                    let instance = get_contract_instance().await;

                    set(&instance, 1, $arg1).await;
                }

                #[tokio::test]
                #[should_panic(expected = "Revert(0)")]
                async fn cant_get_first() {
                    let instance = get_contract_instance().await;

                    let _ = first(&instance).await;
                }

                #[tokio::test]
                #[should_panic(expected = "Revert(0)")]
                async fn cant_get_last() {
                    let instance = get_contract_instance().await;

                    let _ = last(&instance).await;
                }
            }

        }
    }
}
