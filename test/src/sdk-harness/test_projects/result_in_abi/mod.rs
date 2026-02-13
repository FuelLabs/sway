use fuels::{prelude::*, types::Bits256};
use std::str::FromStr;

abigen!(Contract(
    name = "ResultInAbiTestContract",
    abi = "out_for_sdk_harness_tests/result_in_abi-abi.json"
));

async fn get_result_in_abi_instance() -> (ResultInAbiTestContract<Wallet>, ContractId) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out_for_sdk_harness_tests/result_in_abi.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = ResultInAbiTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn test_bool() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok(true);
    let response = contract_methods.bool_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Ok(false);
    let response = contract_methods.bool_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.bool_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u8() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok(42);
    let response = contract_methods.u8_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.u8_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u16() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok(42);
    let response = contract_methods.u16_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.u16_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u32() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok(42);
    let response = contract_methods.u32_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.u32_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_u64() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok(42);
    let response = contract_methods.u64_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.u64_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_b256() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok(Bits256([1u8; 32]));
    let response = contract_methods.b256_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.b256_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_struct() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok(MyStruct {
        first_field: Ok(Address::from_str(
            "0x4242424242424242424242424242424242424242424242424242424242424242",
        )
        .unwrap()),
        second_field: 42,
    });
    let response = contract_methods.struct_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Ok(MyStruct {
        first_field: Err(SomeError::SomeErrorString("error".try_into().unwrap())),
        second_field: 42,
    });
    let response = contract_methods.struct_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.struct_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_tuple() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok((
        Ok(
            Address::from_str("0x4242424242424242424242424242424242424242424242424242424242424242")
                .unwrap(),
        ),
        42,
    ));
    let response = contract_methods.tuple_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Ok((
        Err(SomeError::SomeErrorString("error".try_into().unwrap())),
        42,
    ));
    let response = contract_methods.tuple_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.tuple_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_enum() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok(MyEnum::FirstVariant(Ok(Address::from_str(
        "0x4242424242424242424242424242424242424242424242424242424242424242",
    )
    .unwrap())));
    let response = contract_methods.enum_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Ok(MyEnum::FirstVariant(Err(SomeError::SomeErrorString(
        "error".try_into().unwrap(),
    ))));
    let response = contract_methods.enum_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Ok(MyEnum::SecondVariant(42));
    let response = contract_methods.enum_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.enum_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_array() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok([
        Ok(
            Address::from_str("0x4242424242424242424242424242424242424242424242424242424242424242")
                .unwrap(),
        ),
        Ok(
            Address::from_str("0x6969696969696969696969696969696969696969696969696969696969696969")
                .unwrap(),
        ),
        Ok(
            Address::from_str("0x9999999999999999999999999999999999999999999999999999999999999999")
                .unwrap(),
        ),
    ]);
    let response = contract_methods.array_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Ok([
        Err(SomeError::SomeErrorString("error".try_into().unwrap())),
        Ok(
            Address::from_str("0x6969696969696969696969696969696969696969696969696969696969696969")
                .unwrap(),
        ),
        Err(SomeError::SomeErrorString("error".try_into().unwrap())),
    ]);
    let response = contract_methods.array_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Ok([
        Err(SomeError::SomeErrorString("error".try_into().unwrap())),
        Err(SomeError::SomeErrorString("error".try_into().unwrap())),
        Err(SomeError::SomeErrorString("error".try_into().unwrap())),
    ]);
    let response = contract_methods.array_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.array_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_string() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok("fuel".try_into().unwrap());
    let response = contract_methods.string_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods.string_test(input.clone()).call().await?;
    assert_eq!(input, response.value);

    Ok(())
}

#[tokio::test]
async fn test_option_in_result() -> Result<()> {
    let (instance, _id) = get_result_in_abi_instance().await;
    let contract_methods = instance.methods();

    let input = Ok(Some("fuel".try_into().unwrap()));
    let response = contract_methods
        .option_in_result_test(input.clone())
        .call()
        .await?;
    assert_eq!(input, response.value);

    let input = Ok(None);
    let response = contract_methods
        .option_in_result_test(input.clone())
        .call()
        .await?;
    assert_eq!(input, response.value);

    let input = Err(SomeError::SomeErrorString("error".try_into().unwrap()));
    let response = contract_methods
        .option_in_result_test(input.clone())
        .call()
        .await?;
    assert_eq!(input, response.value);

    Ok(())
}
