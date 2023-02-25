#[macro_export]
macro_rules! testgen {
    (
        // Name of the module to create.
        $module_name:ident,
        // Path to the contract ABI (string literal required for `abigen!`).
        $abi_path:expr,
        // Type to test, as a string literal (required for binary and storage file names).
        $type_label:expr,
        // Type to test, as a Rust type declaration (required for function signatures).
        $type_declaration:ty,
        // Arguments of type `$type_declaration` to use in tests.
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

            // Silences `super::*` warning; required for user-defined types.
            #[allow(unused_imports)]
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

            // Silences `super::*` warning; required for user-defined types.
            #[allow(unused_imports)]
            pub mod wrappers {
                use super::*;

                pub async fn push(instance: &MyContract, value: $type_declaration) {
                    instance.methods()
                        .push(value)
                        .tx_params(TxParameters::new(None, Some(100_000_000), None))
                        .call()
                        .await
                        .unwrap();
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

                pub async fn get(instance: &MyContract, index: u64) -> $type_declaration {
                    instance.methods()
                        .get(index)
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
            }

            // Silences `super::*` warning; required for user-defined types.
            #[allow(unused_imports)]
            pub mod success {
                use super::{
                    *,
                    setup::get_contract_instance,
                    wrappers::*,
                };

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
                async fn can_get() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;

                    assert_eq!(get(&instance, 0).await, $arg0);
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
            }

            // Silences `super::*` warning; required for user-defined types.
            #[allow(unused_imports)]
            pub mod failure {
                use super::{
                    *,
                    setup::get_contract_instance,
                    wrappers::*,
                };

                #[tokio::test]
                #[should_panic(expected = "revert_id: 0")]
                async fn cant_pop() {
                    let instance = get_contract_instance().await;

                    let _ = pop(&instance).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: 0")]
                async fn cant_get() {
                    let instance = get_contract_instance().await;

                    get(&instance, 0).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: 18446744073709486084")]
                async fn cant_remove() {
                    let instance = get_contract_instance().await;

                    let _ = remove(&instance, 0).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: 18446744073709486084")]
                async fn cant_swap_remove() {
                    let instance = get_contract_instance().await;

                    let _ = swap_remove(&instance, 0).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: 18446744073709486084")]
                async fn cant_set() {
                    let instance = get_contract_instance().await;

                    set(&instance, 1, $arg1).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: 18446744073709486084")]
                async fn cant_insert() {
                    let instance = get_contract_instance().await;

                    insert(&instance, 1, $arg1).await;
                }
            }

        }
    }
}
