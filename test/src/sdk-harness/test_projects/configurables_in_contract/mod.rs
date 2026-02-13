use fuels::{prelude::*, types::SizedAsciiString};

// TODO Remove ignore when SDK supports encoding V1 for configurables
// https://github.com/FuelLabs/sway/issues/5727
#[tokio::test]
#[ignore]
async fn contract_uses_default_configurables() -> Result<()> {
    abigen!(Contract(
        name = "MyContract",
        abi =
            "out_for_sdk_harness_tests/configurables_in_contract-abi.json"
    ));

    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let contract_id = Contract::load_from(
        "out_for_sdk_harness_tests/configurables_in_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await?
    .contract_id;

    let contract_instance = MyContract::new(contract_id, wallet.clone());

    let response = contract_instance
        .methods()
        .return_configurables()
        .call()
        .await?;

    let expected_value = (
        8u8,
        true,
        [253u32, 254u32, 255u32],
        "fuel".try_into()?,
        StructWithGeneric {
            field_1: 8u8,
            field_2: 16,
        },
        EnumWithGeneric::VariantOne(true),
        Address::new([0u8; 32]),
        ContractId::new([0u8; 32]),
    );

    assert_eq!(response.value, expected_value);

    Ok(())
}

// TODO Remove ignore when SDK supports encoding V1 for configurables
// https://github.com/FuelLabs/sway/issues/5727
#[tokio::test]
#[ignore]
async fn contract_configurables() -> Result<()> {
    abigen!(Contract(
        name = "MyContract",
        abi =
            "out_for_sdk_harness_tests/configurables_in_contract-abi.json"
    ));

    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let new_str: SizedAsciiString<4> = "FUEL".try_into()?;
    let new_struct = StructWithGeneric {
        field_1: 16u8,
        field_2: 32,
    };
    let new_enum = EnumWithGeneric::VariantTwo;
    let new_address = Address::new([1u8; 32]);
    let new_contract_id = ContractId::new([1u8; 32]);

    let configurables = MyContractConfigurables::default()
        .with_STR_4(new_str.clone())?
        .with_STRUCT(new_struct.clone())?
        .with_ENUM(new_enum.clone())?
        .with_ADDRESS(new_address.clone())?
        .with_MY_CONTRACT_ID(new_contract_id.clone())?;

    let contract_id = Contract::load_from(
        "out_for_sdk_harness_tests/configurables_in_contract.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )?
    .deploy(&wallet, TxPolicies::default())
    .await?
    .contract_id;

    let contract_instance = MyContract::new(contract_id, wallet.clone());

    let response = contract_instance
        .methods()
        .return_configurables()
        .call()
        .await?;

    let expected_value = (
        8u8,
        true,
        [253u32, 254u32, 255u32],
        new_str,
        new_struct,
        new_enum,
        new_address,
        new_contract_id,
    );

    assert_eq!(response.value, expected_value);

    Ok(())
}
