use fuels::{prelude::*, types::SizedAsciiString};

// TODO Remove ignore when SDK supports encoding V1 for configurables
// https://github.com/FuelLabs/sway/issues/5727
#[tokio::test]
#[ignore]
async fn script_uses_default_configurables() -> Result<()> {
    abigen!(Script(
        name = "MyScript",
        abi = "out_for_sdk_harness_tests/configurables_in_script-abi.json"
    ));

    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let bin_path = "out_for_sdk_harness_tests/configurables_in_script.bin";
    let instance = MyScript::new(wallet, bin_path);

    let response = instance.main().call().await?;

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

// TODO Remove ignore when SDK supports encoding V1 for configurables
// https://github.com/FuelLabs/sway/issues/5727
#[tokio::test]
#[ignore]
async fn script_configurables() -> Result<()> {
    abigen!(Script(
        name = "MyScript",
        abi = "out_for_sdk_harness_tests/configurables_in_script-abi.json"
    ));

    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let bin_path = "out_for_sdk_harness_tests/configurables_in_script.bin";
    let instance = MyScript::new(wallet, bin_path);

    let new_str: SizedAsciiString<4> = "FUEL".try_into()?;
    let new_struct = StructWithGeneric {
        field_1: 16u8,
        field_2: 32,
    };
    let new_enum = EnumWithGeneric::VariantTwo;

    let configurables = MyScriptConfigurables::default()
        .with_STR_4(new_str.clone())?
        .with_STRUCT(new_struct.clone())?
        .with_ENUM(new_enum.clone())?;

    let response = instance
        .with_configurables(configurables)
        .main()
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

    pretty_assertions::assert_eq!(response.value, expected_value);

    Ok(())
}
