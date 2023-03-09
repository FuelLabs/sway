use fuels::{prelude::*, types::SizedAsciiString};

#[tokio::test]
async fn contract_uses_default_configurables() -> Result<()> {
    abigen!(Contract(
        name = "MyContract",
        abi =
            "test_projects/configurables_in_contract/out/debug/configurables_in_contract-abi.json"
    ));

    let wallet = launch_provider_and_get_wallet().await;

    let contract_id = Contract::deploy(
        "test_projects/configurables_in_contract/out/debug/configurables_in_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::default(),
    )
    .await?;

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
    );

    assert_eq!(response.value, expected_value);

    Ok(())
}

#[tokio::test]
async fn contract_configurables() -> Result<()> {
    abigen!(Contract(
        name = "MyContract",
        abi =
            "test_projects/configurables_in_contract/out/debug/configurables_in_contract-abi.json"
    ));

    let wallet = launch_provider_and_get_wallet().await;

    let new_str: SizedAsciiString<4> = "FUEL".try_into()?;
    let new_struct = StructWithGeneric {
        field_1: 16u8,
        field_2: 32,
    };
    let new_enum = EnumWithGeneric::VariantTwo;

    let configurables = MyContractConfigurables::new()
        .set_STR_4(new_str.clone())
        .set_STRUCT(new_struct.clone())
        .set_ENUM(new_enum.clone());

    let contract_id = Contract::deploy_with_parameters(
        "test_projects/configurables_in_contract/out/debug/configurables_in_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::default(),
        configurables.into(),
        Salt::default(),
    )
    .await?;

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
    );

    assert_eq!(response.value, expected_value);

    Ok(())
}
