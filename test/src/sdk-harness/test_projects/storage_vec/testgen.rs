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
                abi = $abi_path
            ));

            // Silences `super::*` warning; required for user-defined types.
            #[allow(unused_imports)]
            pub mod setup {
                use super::*;

                pub async fn get_contract_instance() -> MyContract<Wallet> {
                    let wallet = launch_provider_and_get_wallet().await.unwrap();
                    let id = Contract::load_from(
                        &format!(
                            "out/svec_{}.bin",
                            $type_label,
                        ),
                        LoadConfiguration::default()
                        .with_storage_configuration(StorageConfiguration::default()
                            .add_slot_overrides_from_file(
                                &format!(
                                    "out/svec_{}-storage_slots.json",
                                    $type_label,
                                )
                            )
                        .unwrap()),
                    )
                    .unwrap()
                    .deploy(&wallet, TxPolicies::default())
                    .await
                    .unwrap()
                    .contract_id;

                    MyContract::new(id.clone(), wallet)
                }
            }

            // Silences `super::*` warning; required for user-defined types.
            #[allow(unused_imports)]
            pub mod wrappers {
                use super::*;

                pub async fn push(instance: &MyContract<Wallet>, value: $type_declaration) {
                    instance.methods()
                        .push(value)
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn pop(instance: &MyContract<Wallet>) -> $type_declaration {
                    instance.methods()
                        .pop()
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn get(instance: &MyContract<Wallet>, index: u64) -> $type_declaration {
                    instance.methods()
                        .get(index)
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn remove(instance: &MyContract<Wallet>, index: u64) -> $type_declaration {
                    instance.methods()
                        .remove(index)
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn swap_remove(instance: &MyContract<Wallet>, index: u64) -> $type_declaration {
                    instance.methods()
                        .swap_remove(index)
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn set(instance: &MyContract<Wallet>, index: u64, value: $type_declaration) {
                    instance.methods()
                        .set(index, value)
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn insert(instance: &MyContract<Wallet>, index: u64, value: $type_declaration) {
                    instance.methods()
                        .insert(index, value)
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn len(instance: &MyContract<Wallet>) -> u64 {
                    instance.methods()
                        .len()
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn is_empty(instance: &MyContract<Wallet>) -> bool {
                    instance.methods()
                        .is_empty()
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn clear(instance: &MyContract<Wallet>) {
                    instance.methods()
                        .clear()
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn swap(instance: &MyContract<Wallet>, index_0: u64, index_1: u64) {
                    instance.methods()
                        .swap(index_0, index_1)
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn first(instance: &MyContract<Wallet>) -> $type_declaration {
                    instance.methods()
                        .first()
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn last(instance: &MyContract<Wallet>) -> $type_declaration {
                    instance.methods()
                        .last()
                        .call()
                        .await
                        .unwrap()
                        .value
                }

                pub async fn reverse(instance: &MyContract<Wallet>) {
                    instance.methods()
                        .reverse()
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn fill(instance: &MyContract<Wallet>, value: $type_declaration) {
                    instance.methods()
                        .fill(value)
                        .call()
                        .await
                        .unwrap();
                }

                pub async fn resize(instance: &MyContract<Wallet>, new_len: u64, value: $type_declaration) {
                    instance.methods()
                        .resize(new_len, value)
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
                    assert_eq!(get(&instance, 0).await, $arg0);
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

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);

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

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);

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

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);

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

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);

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

                    assert!(!is_empty(&instance).await);

                    clear(&instance).await;

                    assert!(is_empty(&instance).await);

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    assert!(!is_empty(&instance).await);

                    clear(&instance).await;

                    assert!(is_empty(&instance).await);
                }

                #[tokio::test]
                async fn can_swap() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);
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

                    assert_eq!(len(&instance).await, 2);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);

                    assert_eq!(first(&instance).await, $arg0);
                }

                #[tokio::test]
                async fn can_get_last() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;

                    assert_eq!(len(&instance).await, 2);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(last(&instance).await, $arg1);
                }

                #[tokio::test]
                async fn can_reverse_even_len() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);

                    reverse(&instance).await;

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg3);
                    assert_eq!(get(&instance, 1).await, $arg2);
                    assert_eq!(get(&instance, 2).await, $arg1);
                    assert_eq!(get(&instance, 3).await, $arg0);

                    reverse(&instance).await;

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);
                }

                #[tokio::test]
                async fn can_reverse_odd_len() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;

                    assert_eq!(len(&instance).await, 3);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);

                    reverse(&instance).await;

                    assert_eq!(len(&instance).await, 3);

                    assert_eq!(get(&instance, 0).await, $arg2);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg0);

                    reverse(&instance).await;

                    assert_eq!(len(&instance).await, 3);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                }

                #[tokio::test]
                async fn can_fill() {
                    let instance = get_contract_instance().await;

                    push(&instance, $arg0).await;
                    push(&instance, $arg1).await;
                    push(&instance, $arg2).await;
                    push(&instance, $arg3).await;

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);

                    fill(&instance, $arg4).await;

                    assert_eq!(len(&instance).await, 4);
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

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);

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

                    assert_eq!(len(&instance).await, 4);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
                    assert_eq!(get(&instance, 2).await, $arg2);
                    assert_eq!(get(&instance, 3).await, $arg3);

                    resize(&instance, 2, $arg4).await;

                    assert_eq!(len(&instance).await, 2);
                    assert_eq!(get(&instance, 0).await, $arg0);
                    assert_eq!(get(&instance, 1).await, $arg1);
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
                #[should_panic(expected = "revert_id: Some(0)")]
                async fn cant_pop() {
                    let instance = get_contract_instance().await;

                    let _ = pop(&instance).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: Some(0)")]
                async fn cant_get() {
                    let instance = get_contract_instance().await;

                    get(&instance, 0).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: Some(18446744073709486084)")]
                async fn cant_remove() {
                    let instance = get_contract_instance().await;

                    let _ = remove(&instance, 0).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: Some(18446744073709486084)")]
                async fn cant_swap_remove() {
                    let instance = get_contract_instance().await;

                    let _ = swap_remove(&instance, 0).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: Some(18446744073709486084)")]
                async fn cant_set() {
                    let instance = get_contract_instance().await;

                    set(&instance, 1, $arg1).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: Some(18446744073709486084)")]
                async fn cant_insert() {
                    let instance = get_contract_instance().await;

                    insert(&instance, 1, $arg1).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: Some(0)")]
                async fn cant_get_first() {
                    let instance = get_contract_instance().await;

                    let _ = first(&instance).await;
                }

                #[tokio::test]
                #[should_panic(expected = "revert_id: Some(0)")]
                async fn cant_get_last() {
                    let instance = get_contract_instance().await;

                    let _ = last(&instance).await;
                }


                #[tokio::test]
                #[should_panic(expected = "revert_id: Some(18446744073709486084)")]
                async fn cant_swap() {
                    let instance = get_contract_instance().await;

                    let _ = swap(&instance, 0, 1).await;
                }
            }

        }
    }
}
