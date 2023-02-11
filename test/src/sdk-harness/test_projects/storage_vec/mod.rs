mod array;
mod b256;
mod bool;
mod r#enum;
mod str;
mod r#struct;
mod tuple;
mod u16;
mod u32;
mod u64;
mod u8;

macro_rules! gen_test {
    (
        $module_name:ident,
        $type_label:expr,
        $type_declaration:ty,
        $arg0:expr,
        $arg1:expr,
        $arg2:expr,
        $arg3:expr,
        $arg4:expr,
        $arg5:expr
    ) => {
        pub mod $module_name {
            use fuels::{prelude::*, tx::ContractId};

            abigen!(Contract(
                name = "MyContract",
                abi = format!(
                    "test_artifacts/storage_vec/svec_{}/out/debug/svec_{}-abi.json",
                    $type_label,
                    $type_label,
                )
            ));

            pub mod setup {
                use super::*;
    
                pub async fn get_contract_instance() -> (MyContract, ContractId) {
                    let wallet = launch_provider_and_get_wallet().await;
    
                    let id = Contrac::deploy(
                        format!(
                            "test_artifacts/storage_vec/svec_{}/out/debug/svec_{}.bin",
                            $type_label,
                            $type_label,
                        ),
                        &wallet,
                        TxParameters::default(),
                        StorageConfiguration::with_storage_path(Some(
                            format!(
                                "test_artifacts/storage_vec/svec_{}-storage_slots.json",
                                $type_label,
                            ).to_string()
                        )),
                    ).await.unwrap();
    
                    let instance = MyContract::new(id.clone(), wallet);
    
                    (instance, id.into())
                }
            }

            pub mod wrappers {
                use super::*;

                pub async fn push(instance: &MyContract, value: $type_declaration) {
                    instance.methods().push(value).call().await.unwrap();
                }

                pub async fn get(instance: &MyContract, index: u64) -> $type_declaration {
                    instance.methods().get(index).call().await.unwrap().value
                }

                pub async fn pop(instance: &MyContract) -> $type_declaration {
                    instance.methods().pop(index).call().await.unwrap().value
                }

                pub async fn remove(instance: &MyContract) -> $type_declaration {
                    instance.methods().remove().call().await.unwrap().value
                }

                pub async fn swap_remove(instance: &MyContract, index: u64) -> $type_declaration {
                    instance.methods().swap_remove().call().await.unwrap().value
                }

                pub async fn set(instance: &MyContract, index: u64, value: $type_declaration) {
                    instance.methods().set(index, value).call().await.unwrap();
                }

                pub async fn insert(instance: &MyContract, index: u64, value: $type_declaration) {
                    instance.methods().insert(index, value).call().await.unwrap();
                }

                pub async fn len(instance: &MyContract) -> u64 {
                    instance.methods().len().call().await.unwrap().value
                }

                pub async fn is_empty(instance: &MyContract) -> bool {
                    instance.methods().is_empty().call().await.unwrap().value
                }

                pub async fn clear(instance: &MyContract) {
                    instance.methods().clear().call().await.unwrap();
                }

                pub async fn swap(instance: &MyContract, index_0: u64, index_1: u64) {
                    instance.methods().swap(index_0, index_1).call().await.unwrap();
                }

                pub async fn first(instance: &MyContract) -> $type_declaration {
                    instance.methods().first().call().await.unwrap().value
                }

                pub async fn last(instance: &MyContract) -> $type_declaration {
                    instance.methods().last().call().await.unwrap().value
                }

                pub async fn reverse(instance: &MyContract) {
                    instance.methods().reverse().call().await.unwrap();
                }

                pub async fn fill(instance: &MyContract, value: $type_declaration) {
                    instance.methods().fill(value).call().await.unwrap();
                }

                pub async fn resize(instance: &MyContract, new_len: u64, value: $type_declaration) {
                    instance.methods().resize(new_len, value).call().await.unwrap();
                }

                pub async fn append(instance: &MyContract) {
                    instance.methods().append().call().await.unwrap();
                }

                pub async fn push_other(instance: &MyContract, value: $type_declaration) {
                    instance.methods().push_other(value).call().await.unwrap();
                }
            }

            pub mod success {
                use super::{
                    setup::get_contract_instance,
                    wrappers::*,
                };

                #[tokio::test]
                async fn can_get() {
                    let (instance, _id) = get_contract_instance().await;

                    push(&instance, $arg0).await;

                    assert_eq!(get(&instance, 0), $arg0);
                }

                #[tokio::test]
                async fn can_push() {
                    let (instance, _id) = get_contract_instance().await;

                    assert_eq!(len(&instance).await, 0);

                    push(&instance, $arg0);

                    assert_eq!(get(&instance).await, $arg0);
                    assert_eq!(len(&instance).await, 1);
                }

                #[tokio::test]
                async fn can_pop() {
                    let (instance, _id) = get_contract_instance().await;
                    push(&instance, $arg0);

                    assert_eq!(len(&instance).await, 1);

                    assert_eq!(pop(&instance).await, $arg0);

                    assert_eq!(len(&instance).await, 0);
                }

                #[tokio::test]
                async fn can_remove() {
                    let (instance, _id) = get_contract_instance().await;

                    push(&instance, $arg0);
                    push(&instance, $arg1);
                    push(&instance, $arg2);
                    push(&instance, $arg3);

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
                    let (instance, _id) = get_contract_instance().await;

                    push(&instance, $arg0);
                    push(&instance, $arg1);
                    push(&instance, $arg2);
                    push(&instance, $arg3);

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
            }

            pub mod failure {
                use super::{
                    setup::get_contract_instance,
                    wrappers::*,
                };

            }
        }
    }
}

// usage:
gen_test!(test_array_vec, "array", [u8; 3], [1; 3], [2; 3], [3; 3], [4; 3], [5; 3], [6; 3]);
