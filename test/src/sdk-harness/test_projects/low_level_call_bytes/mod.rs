use fuel_vm::fuel_tx::{Bytes32, ContractId, Output, TxPointer, UtxoId};
use fuels::{
    accounts::wallet::WalletUnlocked,
    prelude::*,
    types::{input::Input, Bits256, SizedAsciiString},
};

macro_rules! fn_selector {
    ( $fn_name: ident ( $($fn_arg: ty),* )  ) => {
         ::fuels::core::codec::resolve_fn_selector(stringify!($fn_name), &[$( <$fn_arg as ::fuels::core::traits::Parameterize>::param_type() ),*]).to_vec()
    }
}
macro_rules! calldata {
    ( $($arg: expr),* ) => {
        ::fuels::core::codec::ABIEncoder::encode(&[$(::fuels::core::traits::Tokenizable::into_token($arg)),*]).unwrap().resolve(0)
    }
}

// Load abi from json
abigen!(
    Contract(
        name = "TestContract",
        abi = "test_artifacts/low_level_callee_contract/out/debug/test_contract-abi.json"
    ),
    Script(
        name = "TestScript",
        abi = "test_projects/low_level_call_bytes/out/debug/test_script_bytes-abi.json"
    )
);

async fn low_level_call(
    id: ContractId,
    wallet: WalletUnlocked,
    function_selector: Vec<u8>,
    calldata: Vec<u8>,
    single_value_type_arg: bool,
) {
    // Build the script instance
    let script_instance = TestScript::new(
        wallet,
        "test_projects/low_level_call_bytes/out/debug/test_script_bytes.bin",
    );

    // Add the contract being called to the inputs and outputs
    let contract_input = Input::Contract {
        utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
        tx_pointer: TxPointer::default(),
        contract_id: id,
    };

    let contract_output = Output::Contract {
        input_index: 0u8,
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
    };

    // Run the script which will call the contract
    let tx = script_instance
        .main(
            id,
            fuels::types::Bytes(function_selector),
            fuels::types::Bytes(calldata),
            single_value_type_arg,
        )
        .with_inputs(vec![contract_input])
        .with_outputs(vec![contract_output])
        .tx_params(TxParameters::default().with_gas_limit(10_000_000));

    tx.call().await.unwrap();
}

async fn get_contract_instance() -> (TestContract<WalletUnlocked>, ContractId, WalletUnlocked) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();

    let id = Contract::load_from(
        "test_artifacts/low_level_callee_contract/out/debug/test_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    let instance = TestContract::new(id.clone(), wallet.clone());

    (instance, id.into(), wallet)
}

#[tokio::test]
async fn can_call_with_one_word_arg() {
    let (instance, id, wallet) = get_contract_instance().await;

    let function_selector = fn_selector!(set_value(u64));

    let calldata = calldata!(42u64);

    // Calling "set_value(u64)" with argument "42" should set the value to 42
    low_level_call(id, wallet, function_selector, calldata, true).await;
    let result = instance.methods().get_value().call().await.unwrap().value;
    assert_eq!(result, 42);
}

#[tokio::test]
async fn can_call_with_multi_word_arg() {
    let (instance, id, wallet) = get_contract_instance().await;

    let function_selector = fn_selector!(set_b256_value(Bits256));

    let calldata = calldata!(Bits256([1u8; 32]));

    low_level_call(id, wallet, function_selector, calldata, false).await;
    let result = instance
        .methods()
        .get_b256_value()
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(result, Bits256([1u8; 32]));
}

#[tokio::test]
async fn can_call_with_multiple_args() {
    let (instance, id, wallet) = get_contract_instance().await;

    let function_selector = fn_selector!(set_value_multiple(u64, u64));
    let calldata = calldata!(23u64, 42u64);

    low_level_call(id, wallet, function_selector, calldata, false).await;
    let result = instance.methods().get_value().call().await.unwrap().value;
    assert_eq!(result, 23 + 42);
}

#[tokio::test]
async fn can_call_with_multiple_args_complex() {
    let (instance, id, wallet) = get_contract_instance().await;

    let function_selector =
        fn_selector!(set_value_multiple_complex(MyStruct, SizedAsciiString::<4>));
    let calldata = calldata!(
        MyStruct {
            a: true,
            b: [1, 2, 3],
        },
        SizedAsciiString::<4>::try_from("fuel").unwrap()
    );

    low_level_call(id, wallet, function_selector, calldata, false).await;

    let result_uint = instance.methods().get_value().call().await.unwrap().value;
    let result_bool = instance
        .methods()
        .get_bool_value()
        .call()
        .await
        .unwrap()
        .value;
    let result_str = instance
        .methods()
        .get_str_value()
        .call()
        .await
        .unwrap()
        .value;

    assert_eq!(result_uint, 2);
    assert!(result_bool);
    assert_eq!(result_str, "fuel");
}
