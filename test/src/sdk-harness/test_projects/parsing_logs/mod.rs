use fuels::prelude::*;

abigen!(
    ParsingLogsTestContract,
    "test_projects/parsing_logs/out/debug/parsing_logs-abi.json"
);

async fn get_parsing_logs_instance() -> (ParsingLogsTestContract, ContractId) {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/parsing_logs/out/debug/parsing_logs.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/parsing_logs/out/debug/parsing_logs-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();
    let instance = ParsingLogsTestContract::new(id.to_string(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn test_parse_logged_varibles() -> Result<(), Error> {
    let (instance, _id) = get_parsing_logs_instance().await;

    let contract_methods = instance.methods();
    let response = contract_methods.produce_logs_variables().call().await?;

    let log_u64 = instance.logs_with_type::<u64>(&response.receipts)?;
    let log_bits256 = instance.logs_with_type::<Bits256>(&response.receipts)?;
    let log_string = instance.logs_with_type::<SizedAsciiString<4>>(&response.receipts)?;
    let log_array = instance.logs_with_type::<[u8; 3]>(&response.receipts)?;

    let expected_bits256 = Bits256([
        239, 134, 175, 169, 105, 108, 240, 220, 99, 133, 226, 196, 7, 166, 225, 89, 161, 16, 60,
        239, 183, 226, 174, 6, 54, 251, 51, 211, 203, 42, 158, 74,
    ]);

    assert_eq!(log_u64, vec![64]);
    assert_eq!(log_bits256, vec![expected_bits256]);
    assert_eq!(log_string, vec!["Fuel"]);
    assert_eq!(log_array, vec![[1, 2, 3]]);

    Ok(())
}

#[tokio::test]
async fn test_parse_logs_values() -> Result<(), Error> {
    let (instance, _id) = get_parsing_logs_instance().await;

    let contract_methods = instance.methods();
    let response = contract_methods.produce_logs_values().call().await?;

    let log_u64 = instance.logs_with_type::<u64>(&response.receipts)?;
    let log_u32 = instance.logs_with_type::<u32>(&response.receipts)?;
    let log_u16 = instance.logs_with_type::<u16>(&response.receipts)?;
    let log_u8 = instance.logs_with_type::<u8>(&response.receipts)?;
    // try to retrieve non existent log
    let log_nonexistent = instance.logs_with_type::<bool>(&response.receipts)?;

    assert_eq!(log_u64, vec![64]);
    assert_eq!(log_u32, vec![32]);
    assert_eq!(log_u16, vec![16]);
    assert_eq!(log_u8, vec![8]);
    assert!(log_nonexistent.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_parse_logs_custom_types() -> Result<(), Error> {
    let (instance, _id) = get_parsing_logs_instance().await;

    let contract_methods = instance.methods();
    let response = contract_methods.produce_logs_custom_types().call().await?;

    let log_test_struct = instance.logs_with_type::<TestStruct>(&response.receipts)?;
    let log_test_enum = instance.logs_with_type::<TestEnum>(&response.receipts)?;

    let expected_bits256 = Bits256([
        239, 134, 175, 169, 105, 108, 240, 220, 99, 133, 226, 196, 7, 166, 225, 89, 161, 16, 60,
        239, 183, 226, 174, 6, 54, 251, 51, 211, 203, 42, 158, 74,
    ]);
    let expected_struct = TestStruct {
        field_1: true,
        field_2: expected_bits256,
        field_3: 64,
    };
    let expected_enum = TestEnum::VariantTwo();

    assert_eq!(log_test_struct, vec![expected_struct]);
    assert_eq!(log_test_enum, vec![expected_enum]);

    Ok(())
}

#[tokio::test]
async fn test_parse_logs_generic_types() -> Result<(), Error> {
    let (instance, _id) = get_parsing_logs_instance().await;

    let contract_methods = instance.methods();
    let response = contract_methods.produce_logs_generic_types().call().await?;

    let log_struct = instance.logs_with_type::<StructWithGeneric<[_; 3]>>(&response.receipts)?;
    let log_enum = instance.logs_with_type::<EnumWithGeneric<[_; 3]>>(&response.receipts)?;
    let log_struct_nested = instance
        .logs_with_type::<StructWithNestedGeneric<StructWithGeneric<[_; 3]>>>(&response.receipts)?;
    let log_struct_deeply_nested = instance.logs_with_type::<StructDeeplyNestedGeneric<
        StructWithNestedGeneric<StructWithGeneric<[_; 3]>>,
    >>(&response.receipts)?;

    let l = [1u8, 2u8, 3u8];
    let expected_struct = StructWithGeneric {
        field_1: l,
        field_2: 64,
    };
    let expected_enum = EnumWithGeneric::VariantOne(l);
    let expected_nested_struct = StructWithNestedGeneric {
        field_1: expected_struct.clone(),
        field_2: 64,
    };
    let expected_deeply_nested_struct = StructDeeplyNestedGeneric {
        field_1: expected_nested_struct.clone(),
        field_2: 64,
    };

    assert_eq!(log_struct, vec![expected_struct]);
    assert_eq!(log_enum, vec![expected_enum]);
    assert_eq!(log_struct_nested, vec![expected_nested_struct]);
    assert_eq!(
        log_struct_deeply_nested,
        vec![expected_deeply_nested_struct]
    );

    Ok(())
}

#[tokio::test]
async fn test_fetch_logs() -> Result<(), Error> {
    let (instance, _id) = get_parsing_logs_instance().await;

    let contract_methods = instance.methods();
    let response = contract_methods.produce_multiple_logs().call().await?;
    let logs = instance.fetch_logs(&response.receipts);

    let expected_bits256 = Bits256([
        239, 134, 175, 169, 105, 108, 240, 220, 99, 133, 226, 196, 7, 166, 225, 89, 161, 16, 60,
        239, 183, 226, 174, 6, 54, 251, 51, 211, 203, 42, 158, 74,
    ]);
    let expected_struct = TestStruct {
        field_1: true,
        field_2: expected_bits256,
        field_3: 64,
    };
    let expected_enum = TestEnum::VariantTwo();
    let expected_generic_struct = StructWithGeneric {
        field_1: expected_struct.clone(),
        field_2: 64,
    };
    let expected_logs: Vec<String> = vec![
        format!("{:#?}", 64u64),
        format!("{:#?}", 32u32),
        format!("{:#?}", 16u16),
        format!("{:#?}", 8u8),
        format!("{:#?}", 64u64),
        format!("{:#?}", expected_bits256),
        format!("{:#?}", SizedAsciiString::<4>::new("Fuel".to_string())?),
        format!("{:#?}", [1, 2, 3]),
        format!("{:#?}", expected_struct),
        format!("{:#?}", expected_enum),
        format!("{:#?}", expected_generic_struct),
    ];

    assert_eq!(logs, expected_logs);

    Ok(())
}
