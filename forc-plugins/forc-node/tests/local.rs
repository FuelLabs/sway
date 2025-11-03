use std::{
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use forc_client::{cmd, op::call as forc_call, NodeTarget};
use forc_node::local::{cmd::LocalCmd, run};
use fuel_core::service::FuelService;
use fuel_core_client::client::FuelClient;
use fuel_crypto::SecretKey;
use fuel_tx::Input;
use fuels::{
    accounts::{signers::private_key::PrivateKeySigner, wallet::Wallet},
    prelude::{
        setup_single_asset_coins, setup_test_provider, AssetId, Contract, LoadConfiguration,
        NodeConfig, Provider, TxPolicies,
    },
};
use serde_json::json;
use tokio::time::sleep;

const DEFAULT_PRIVATE_KEY: &str =
    "0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c";

mod fork {
    pub const CONTRACT_BIN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fork/fork.bin");
    pub const CONTRACT_ABI: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fork/fork-abi.json");
    fuels::prelude::abigen!(Contract(
        name = "Contract",
        abi = "forc-plugins/forc-node/tests/fork/fork-abi.json",
    ));
}

mod fork_caller {
    pub const CONTRACT_BIN: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fork-caller/fork-caller.bin"
    );
    pub const CONTRACT_ABI: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fork-caller/fork-caller-abi.json"
    );
    fuels::prelude::abigen!(Contract(
        name = "Contract",
        abi = "forc-plugins/forc-node/tests/fork-caller/fork-caller-abi.json",
    ));
}

async fn run_node(fork_url: Option<String>) -> (FuelService, String) {
    let port = portpicker::pick_unused_port().expect("pick port");

    let db_path = tempfile::tempdir()
        .expect("Failed to create temp dir for tests")
        .path()
        .to_path_buf();

    let secret = SecretKey::from_str(DEFAULT_PRIVATE_KEY).expect("valid private key");
    let address = Input::owner(&secret.public_key());
    let default_account_address = format!("{address:#x}");

    let local_cmd = LocalCmd {
        chain_config: None,
        port: Some(port),
        db_path: Some(db_path),
        account: vec![default_account_address],
        db_type: Some(fuel_core::service::DbType::RocksDb),
        debug: true,
        historical_execution: true,
        poa_instant: true,
        fork_url,
        fork_block_number: None,
        non_interactive: true,
    };
    let service = run(local_cmd, false).await.unwrap().unwrap();
    // Wait for node to start graphql service
    sleep(Duration::from_secs(2)).await;

    (service, format!("http://127.0.0.1:{port}/v1/graphql"))
}

async fn forc_call_result(
    contract_id: &fuel_tx::ContractId,
    abi_path: &str,
    graphql_endpoint: &str,
    function: &str,
    function_args: Vec<String>,
) -> String {
    let secret_key = SecretKey::from_str(DEFAULT_PRIVATE_KEY).expect("parse private key");
    let call_cmd = cmd::Call {
        address: (*(*contract_id)).into(),
        abi: Some(cmd::call::AbiSource::File(PathBuf::from(abi_path))),
        function: Some(function.to_string()),
        function_args,
        node: NodeTarget {
            node_url: Some(graphql_endpoint.to_string()),
            ..Default::default()
        },
        caller: cmd::call::Caller {
            signing_key: Some(secret_key),
            wallet: false,
        },
        call_parameters: Default::default(),
        mode: cmd::call::ExecutionMode::DryRun,
        gas: None,
        external_contracts: None,
        contract_abis: None,
        label: None,
        output: cmd::call::OutputFormat::Raw,
        list_functions: false,
        variable_output: None,
        verbosity: 0,
        debug: false,
    };

    let operation = call_cmd
        .validate_and_get_operation()
        .expect("validate forc-call operation");

    forc_call::call(operation, call_cmd)
        .await
        .expect("forc-call failed")
        .result
        .expect("forc-call did not return result")
        .trim()
        .to_string()
}

#[tokio::test]
async fn fork_contract_bytecode() {
    let secret_key = SecretKey::from_str(DEFAULT_PRIVATE_KEY).expect("parse private key");
    let signer = PrivateKeySigner::new(secret_key);
    let coins = setup_single_asset_coins(signer.address(), AssetId::zeroed(), 1, 1_000_000);
    let node_config = NodeConfig {
        addr: SocketAddr::new(
            Ipv4Addr::LOCALHOST.into(),
            portpicker::pick_unused_port().expect("pick port"),
        ),
        ..Default::default()
    };

    let provider = setup_test_provider(coins, vec![], Some(node_config), None)
        .await
        .expect("start test provider");
    let graphql_endpoint = format!("{}/v1/graphql", provider.url());
    let wallet = Wallet::new(signer, provider.clone());

    let deployed = Contract::load_from(Path::new(fork::CONTRACT_BIN), LoadConfiguration::default())
        .expect("load contract bytecode")
        .deploy(&wallet, TxPolicies::default())
        .await
        .expect("deploy contract");
    let contract_id_hex = format!("{:#x}", deployed.contract_id);

    let original_client =
        FuelClient::new(&graphql_endpoint).expect("create fuel client for original node");
    let original_contract = original_client
        .contract(&deployed.contract_id)
        .await
        .expect("query original node")
        .expect("contract missing on original node");
    assert!(
        !original_contract.bytecode.is_empty(),
        "bytecode missing on original node"
    );

    let (_fork_service, fork_url) = run_node(Some(graphql_endpoint)).await;
    let graphql_query = format!("{{ contract(id: \"{contract_id_hex}\") {{ bytecode }} }}");
    let response = reqwest::Client::new()
        .post(&fork_url)
        .header("Content-Type", "application/json")
        .json(&json!({ "query": graphql_query }))
        .send()
        .await
        .expect("query test provider for bytecode");
    assert!(
        response.status().is_success(),
        "graphql request failed: {}",
        response.status()
    );

    let body: serde_json::Value = response.json().await.expect("parse graphql response");
    if let Some(errors) = body.get("errors") {
        panic!("graphql reported errors: {errors}");
    }

    let bytecode_value = body
        .get("data")
        .and_then(|data| data.get("contract"))
        .and_then(|contract| contract.get("bytecode"))
        .expect("missing bytecode in graphql response");

    let serde_json::Value::String(bytecode) = bytecode_value else {
        panic!("bytecode missing for deployed contract");
    };

    assert!(
        !bytecode.trim().is_empty(),
        "bytecode missing for forked contract"
    );
}

#[tokio::test]
async fn fork_contract_state_with_forc_call() {
    let secret_key = SecretKey::from_str(DEFAULT_PRIVATE_KEY).expect("parse private key");
    let signer = PrivateKeySigner::new(secret_key);

    // deploy a contract on the original node
    let (_original_service, original_url) = run_node(None).await;
    let wallet = Wallet::new(
        signer.clone(),
        Provider::connect(&original_url)
            .await
            .expect("connect provider to original node"),
    );

    let deployment =
        Contract::load_from(Path::new(fork::CONTRACT_BIN), LoadConfiguration::default())
            .expect("load contract bytecode")
            .deploy(&wallet, TxPolicies::default())
            .await
            .expect("deploy contract to original node");
    let contract_id = deployment.contract_id;

    // verify that the original node returns the original contract state (remains unchanged)
    let original_count = forc_call_result(
        &contract_id,
        fork::CONTRACT_ABI,
        &original_url,
        "get_count",
        vec![],
    )
    .await;
    assert_eq!(
        original_count, "0",
        "unexpected initial contract state on original node"
    );

    // fork the original node
    let (_fork_service, fork_url) = run_node(Some(original_url.clone())).await;

    // verify that the forked node returns the original contract state (remains unchanged)
    let fork_result = forc_call_result(
        &contract_id,
        fork::CONTRACT_ABI,
        &fork_url,
        "get_count",
        vec![],
    )
    .await;
    assert_eq!(
        fork_result, original_count,
        "forked node returned unexpected contract state"
    );

    // update the contract state on the original node
    let increment_amount_orig = (2u64, 3u64);
    let contract_instance = fork::Contract::new(contract_id, wallet);
    contract_instance
        .methods()
        .increment_count(fork::Adder {
            vals: increment_amount_orig,
        })
        .call()
        .await
        .expect("increment contract counter");

    // verify that the contract state was updated on the original node
    let updated_count = contract_instance
        .methods()
        .get_count()
        .call()
        .await
        .expect("call get_count after update")
        .value;
    assert_eq!(
        updated_count,
        increment_amount_orig.0 + increment_amount_orig.1,
        "failed to update contract state"
    );

    // verify that the forked node returns the original contract state (remains unchanged)
    let fork_result = forc_call_result(
        &contract_id,
        fork::CONTRACT_ABI,
        &fork_url,
        "get_count",
        vec![],
    )
    .await;
    assert_eq!(
        fork_result, original_count,
        "forked node returned unexpected contract state"
    );

    let wallet = Wallet::new(
        signer,
        Provider::connect(&fork_url)
            .await
            .expect("connect provider to forked node"),
    );

    // update the contract state on the forked node
    let increment_amount_fork = (4u64, 5u64);
    let contract_instance = fork::Contract::new(contract_id, wallet);
    contract_instance
        .methods()
        .increment_count(fork::Adder {
            vals: increment_amount_fork,
        })
        .call()
        .await
        .expect("increment contract counter");

    // verify that the contract state was updated on the forked node
    let fork_result = forc_call_result(
        &contract_id,
        fork::CONTRACT_ABI,
        &fork_url,
        "get_count",
        vec![],
    )
    .await;
    assert_eq!(
        fork_result,
        format!("{}", increment_amount_fork.0 + increment_amount_fork.1),
        "forked node returned unexpected contract state"
    );

    // verify that the original node returns the original contract state (remains unchanged)
    let original_result = forc_call_result(
        &contract_id,
        fork::CONTRACT_ABI,
        &original_url,
        "get_count",
        vec![],
    )
    .await;
    assert_eq!(
        original_result,
        format!("{}", increment_amount_orig.0 + increment_amount_orig.1),
        "original node returned unexpected contract state"
    );
}

#[tokio::test]
async fn update_transitive_contract_state() {
    let secret_key = SecretKey::from_str(DEFAULT_PRIVATE_KEY).expect("parse private key");
    let signer = PrivateKeySigner::new(secret_key);

    // deploy contract A on the original node
    let (_original_service, original_url) = run_node(None).await;
    let wallet = Wallet::new(
        signer.clone(),
        Provider::connect(&original_url)
            .await
            .expect("connect provider to original node"),
    );

    // deploy contract A on the original node
    let deployment =
        Contract::load_from(Path::new(fork::CONTRACT_BIN), LoadConfiguration::default())
            .expect("load contract bytecode")
            .deploy(&wallet, TxPolicies::default())
            .await
            .expect("deploy contract to original node");
    let contract_a_id = deployment.contract_id;

    // deploy contract B on the original node
    let deployment = Contract::load_from(
        Path::new(fork_caller::CONTRACT_BIN),
        LoadConfiguration::default(),
    )
    .expect("load contract bytecode")
    .deploy(&wallet, TxPolicies::default())
    .await
    .expect("deploy contract to original node");
    let contract_b_id = deployment.contract_id;

    // verify that the original node returns correct contract A state
    let result = forc_call_result(
        &contract_a_id,
        fork::CONTRACT_ABI,
        &original_url,
        "get_count",
        vec![],
    )
    .await;
    assert_eq!(
        result, "0",
        "original node returned unexpected contract A state"
    );

    // verify that the original node returns correct contract A state via calling contract B
    let result_b = forc_call_result(
        &contract_b_id,
        fork_caller::CONTRACT_ABI,
        &original_url,
        "check_current_count",
        vec![format!("{{{:#x}}}", contract_a_id)],
    )
    .await;
    assert_eq!(
        result_b, "0",
        "unexpected initial contract state on original node"
    );

    // fork the original node
    let (_fork_service, fork_url) = run_node(Some(original_url.clone())).await;

    // verify that the forked node returns the original contract state (remains unchanged)
    let fork_result = forc_call_result(
        &contract_a_id,
        fork::CONTRACT_ABI,
        &fork_url,
        "get_count",
        vec![],
    )
    .await;
    assert_eq!(
        fork_result, "0",
        "forked node returned unexpected contract state"
    );

    // connect to the forked node
    let fork_wallet = Wallet::new(
        signer,
        Provider::connect(&fork_url)
            .await
            .expect("connect provider to forked node"),
    );

    // update the contract A state on the forked node
    let increment_amount_fork = (4u64, 5u64);
    let contract_instance = fork::Contract::new(contract_a_id, fork_wallet.clone());
    contract_instance
        .methods()
        .increment_count(fork::Adder {
            vals: increment_amount_fork,
        })
        .call()
        .await
        .expect("increment contract counter");

    // update the contract A state on the forked node by calling the contract B
    // let contract_instance = fork_caller::Contract::new(contract_b_id.clone(), fork_wallet);
    // contract_instance
    //     .methods()
    //     .call_increment_count(contract_a_id.clone(), fork_caller::Adder { vals: increment_amount_fork })
    //     .with_contract_ids(&[contract_a_id.clone()])
    //     .call()
    //     .await
    //     .expect("fork-caller: update contract A state via contract B");

    // verify that the contract A state was updated on the forked node
    let fork_result = forc_call_result(
        &contract_a_id,
        fork::CONTRACT_ABI,
        &fork_url,
        "get_count",
        vec![],
    )
    .await;
    assert_eq!(
        fork_result,
        format!("{}", increment_amount_fork.0 + increment_amount_fork.1),
        "fork-caller: forked node returned unexpected contract A state"
    );

    // verify that the contract state was transitively updated by calling contract B on the forked node
    let fork_result = forc_call_result(
        &contract_b_id,
        fork_caller::CONTRACT_ABI,
        &fork_url,
        "check_current_count",
        vec![format!("{{{:#x}}}", contract_a_id)],
    )
    .await;
    assert_eq!(
        fork_result,
        format!("{}", increment_amount_fork.0 + increment_amount_fork.1),
        "forked node returned unexpected contract B state"
    );

    // verify that the original node returns unchanged contract A state
    let original_result = forc_call_result(
        &contract_a_id,
        fork::CONTRACT_ABI,
        &original_url,
        "get_count",
        vec![],
    )
    .await;
    assert_eq!(
        original_result, "0",
        "original node returned unexpected contract A state"
    );
}

#[tokio::test]
async fn start_local_node_check_health() {
    let port = portpicker::pick_unused_port().expect("pick port");
    let local_cmd = LocalCmd {
        chain_config: None,
        port: Some(port),
        db_path: None,
        account: vec![],
        db_type: None,
        debug: false,
        historical_execution: false,
        poa_instant: false,
        fork_url: None,
        fork_block_number: None,
        non_interactive: true,
    };

    let _service = run(local_cmd, false).await.unwrap().unwrap();
    // Wait for node to start graphql service
    sleep(Duration::from_secs(2)).await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{port}/v1/graphql"))
        .header("Content-Type", "application/json")
        .json(&json!({
            "query": "{ health }"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");

    assert_eq!(body["data"]["health"], true);
}
