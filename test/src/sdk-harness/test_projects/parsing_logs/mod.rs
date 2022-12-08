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
    let instance = ParsingLogsTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn test_parse_logged_varibles() -> Result<(), Error> {
    let (instance, _id) = get_parsing_logs_instance().await;

    let contract_methods = instance.methods();
    let response = contract_methods.produce_logs_variables().call().await?;

    let log_u64 = response.get_logs_with_type::<u64>()?;
    let log_bits256 = response.get_logs_with_type::<Bits256>()?;
    let log_string = response.get_logs_with_type::<SizedAsciiString<4>>()?;
    let log_array = response.get_logs_with_type::<[u8; 3]>()?;

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

    let log_u64 = response.get_logs_with_type::<u64>()?;
    let log_u32 = response.get_logs_with_type::<u32>()?;
    let log_u16 = response.get_logs_with_type::<u16>()?;
    let log_u8 = response.get_logs_with_type::<u8>()?;
    // try to retrieve non existent log
    let log_nonexistent = response.get_logs_with_type::<bool>()?;

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

    let log_test_struct = response.get_logs_with_type::<TestStruct>()?;
    let log_test_enum = response.get_logs_with_type::<TestEnum>()?;

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

    let log_struct = response.get_logs_with_type::<StructWithGeneric<[_; 3]>>()?;
    let log_enum = response.get_logs_with_type::<EnumWithGeneric<[_; 3]>>()?;
    let log_struct_nested =
        response.get_logs_with_type::<StructWithNestedGeneric<StructWithGeneric<[_; 3]>>>()?;
    let log_struct_deeply_nested = response.get_logs_with_type::<StructDeeplyNestedGeneric<
        StructWithNestedGeneric<StructWithGeneric<[_; 3]>>,
    >>()?;

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
async fn test_get_logs() -> Result<(), Error> {
    let (instance, _id) = get_parsing_logs_instance().await;

    let contract_methods = instance.methods();
    let response = contract_methods.produce_multiple_logs().call().await?;
    let logs = response.get_logs()?;

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
        format!("{:?}", 64u64),
        format!("{:?}", 32u32),
        format!("{:?}", 16u16),
        format!("{:?}", 8u8),
        format!("{:?}", 64u64),
        format!("{:?}", expected_bits256),
        format!("{:?}", SizedAsciiString::<4>::new("Fuel".to_string())?),
        format!("{:?}", [1, 2, 3]),
        format!("{:?}", expected_struct),
        format!("{:?}", expected_enum),
        format!("{:?}", expected_generic_struct),
    ];

    assert_eq!(logs, expected_logs);

    Ok(())
}
