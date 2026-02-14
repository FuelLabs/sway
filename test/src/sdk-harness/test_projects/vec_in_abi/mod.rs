use fuels::{prelude::*, programs::calls::ContractCall, types::Bits256};
use std::str::FromStr;

abigen!(Contract(
    name = "VecInAbiTestContract",
    abi = "out/vec_in_abi-abi.json"
));

async fn get_vec_in_abi_instance() -> (VecInAbiTestContract<Wallet>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out/vec_in_abi.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = VecInAbiTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn test_bool() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![true, false, true];
    let response = contract_methods.bool_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u8() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![42, 43, 44];
    let response = contract_methods.u8_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u16() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![42, 43, 44];
    let response = contract_methods.u16_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u32() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![42, 43, 44];
    let response = contract_methods.u32_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u64() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![42, 43, 44];
    let response = contract_methods.u64_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_b256() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![Bits256([1u8; 32]), Bits256([2u8; 32]), Bits256([3u8; 32])];
    let response = contract_methods.b256_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_struct() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![
        MyStruct {
            first_field: Some(
                Address::from_str(
                    "0x4242424242424242424242424242424242424242424242424242424242424242",
                )
                .unwrap(),
            ),
            second_field: 42,
        },
        MyStruct {
            first_field: None,
            second_field: 43,
        },
        MyStruct {
            first_field: Some(
                Address::from_str(
                    "0x4444444444444444444444444444444444444444444444444444444444444444",
                )
                .unwrap(),
            ),
            second_field: 44,
        },
    ];
    let response = contract_methods.struct_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_enum() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![
        MyEnum::FirstVariant(Some(
            Address::from_str("0x4242424242424242424242424242424242424242424242424242424242424242")
                .unwrap(),
        )),
        MyEnum::FirstVariant(None),
        MyEnum::SecondVariant(42),
    ];
    let response = contract_methods.enum_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_array() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![
        [
            Address::from_str("0x4242424242424242424242424242424242424242424242424242424242424242")
                .unwrap(),
            Address::from_str("0x6969696969696969696969696969696969696969696969696969696969696969")
                .unwrap(),
        ],
        [
            Address::from_str("0x4343434343434343434343434343434343434343434343434343434343434343")
                .unwrap(),
            Address::from_str("0x7070707070707070707070707070707070707070707070707070707070707070")
                .unwrap(),
        ],
        [
            Address::from_str("0x9999999999999999999999999999999999999999999999999999999999999999")
                .unwrap(),
            Address::from_str("0x0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        ],
    ];
    let response = contract_methods.array_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_string() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![
        "fuel".try_into().unwrap(),
        "labs".try_into().unwrap(),
        "rock".try_into().unwrap(),
    ];
    let response = contract_methods.string_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_vec_in_vec() -> Result<()> {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = vec![vec![42, 43, 44], vec![69, 70, 71], vec![99, 100, 101]];
    let response = contract_methods
        .vec_in_vec_test(input.clone())
        .call()
        .await?;
    assert_eq!(
        input.into_iter().flatten().collect::<Vec<_>>(),
        response.value
    );

    Ok(())
}

async fn test_echo<T>(
    f: impl Fn(T) -> CallHandler<fuels::accounts::wallet::Wallet, ContractCall, T>,
    input: T,
) where
    T: Eq
        + Clone
        + fuels::core::traits::Tokenizable
        + fuels::core::traits::Parameterize
        + std::fmt::Debug,
{
    let response = (f)(input.clone()).call().await.unwrap();
    assert_eq!(input, response.value);
}

#[tokio::test]
async fn test_echos() {
    let (instance, _id) = get_vec_in_abi_instance().await;
    let contract_methods = instance.methods();

    test_echo(|v| contract_methods.echo_u8(v), vec![0u8, 1u8, 2u8]).await;
    test_echo(|v| contract_methods.echo_u16(v), vec![0u16, 1u16, 2u16]).await;
    test_echo(|v| contract_methods.echo_u32(v), vec![0u32, 1u32, 2u32]).await;
    test_echo(
        |v| contract_methods.echo_u32_vec_in_vec(v),
        vec![vec![0u32], vec![1u32], vec![2u32]],
    )
    .await;
}
