use fuels::{prelude::*, types::Bits256};
use std::str::FromStr;

abigen!(Contract(
    name = "OptionInAbiTestContract",
    abi = "out_for_sdk_harness_tests/option_in_abi-abi.json"
));

async fn get_option_in_abi_instance() -> (OptionInAbiTestContract<Wallet>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out_for_sdk_harness_tests/option_in_abi.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = OptionInAbiTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn test_bool() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some(true);
    let response = contract_methods.bool_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = Some(false);
    let response = contract_methods.bool_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.bool_test(input).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u8() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some(42);
    let response = contract_methods.u8_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.u8_test(input).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u16() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some(42);
    let response = contract_methods.u16_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.u16_test(input).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u32() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some(42);
    let response = contract_methods.u32_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.u32_test(input).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u64() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some(42);
    let response = contract_methods.u64_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.u64_test(input).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_b256() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some(Bits256([1u8; 32]));
    let response = contract_methods.b256_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.b256_test(input).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_struct() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some(MyStruct {
        first_field: Some(
            Address::from_str("0x4242424242424242424242424242424242424242424242424242424242424242")
                .unwrap(),
        ),
        second_field: 42,
    });
    let response = contract_methods.struct_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Some(MyStruct {
        first_field: None,
        second_field: 42,
    });
    let response = contract_methods.struct_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.struct_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_tuple() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some((
        Some(
            Address::from_str("0x4242424242424242424242424242424242424242424242424242424242424242")
                .unwrap(),
        ),
        42,
    ));
    let response = contract_methods.tuple_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = Some((None, 42));
    let response = contract_methods.tuple_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.tuple_test(input).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_enum() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some(MyEnum::FirstVariant(Some(
        Address::from_str("0x4242424242424242424242424242424242424242424242424242424242424242")
            .unwrap(),
    )));
    let response = contract_methods.enum_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Some(MyEnum::FirstVariant(None));
    let response = contract_methods.enum_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Some(MyEnum::SecondVariant(42));
    let response = contract_methods.enum_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.enum_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_array() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some([
        Some(
            Address::from_str("0x4242424242424242424242424242424242424242424242424242424242424242")
                .unwrap(),
        ),
        Some(
            Address::from_str("0x6969696969696969696969696969696969696969696969696969696969696969")
                .unwrap(),
        ),
        Some(
            Address::from_str("0x9999999999999999999999999999999999999999999999999999999999999999")
                .unwrap(),
        ),
    ]);
    let response = contract_methods.array_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = Some([
        None,
        Some(
            Address::from_str("0x6969696969696969696969696969696969696969696969696969696969696969")
                .unwrap(),
        ),
        None,
    ]);
    let response = contract_methods.array_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = Some([None, None, None]);
    let response = contract_methods.array_test(input).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.array_test(input).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_string() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some("fuel".try_into().unwrap());
    let response = contract_methods.string_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods.string_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_result_in_option() -> Result<()> {
    let (instance, _id) = get_option_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Some(Ok("fuel".try_into().unwrap()));
    let response = contract_methods
        .result_in_option_test(input.clone())
        .call()
        .await?;
    assert_eq!(input, response.value);

    let input = Some(Err(SomeError::SomeErrorString("error".try_into().unwrap())));
    let response = contract_methods
        .result_in_option_test(input.clone())
        .call()
        .await?;
    assert_eq!(input, response.value);

    let input = None;
    let response = contract_methods
        .result_in_option_test(input.clone())
        .call()
        .await?;
    assert_eq!(input, response.value);

    Ok(())
}
