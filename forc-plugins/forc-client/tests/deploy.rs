use forc::cli::shared::Pkg;
use forc_client::{
    cmd,
    op::{deploy, DeployedContract, DeployedExecutable, DeployedPackage},
    util::{account::ForcClientAccount, tx::update_proxy_contract_target},
    NodeTarget,
};
use forc_pkg::manifest::Proxy;
use fuel_crypto::SecretKey;
use fuel_tx::{ContractId, Salt};
use fuels::{
    macros::abigen,
    types::{transaction::TxPolicies, AsciiString, Bits256, SizedAsciiString},
};
use fuels_accounts::{
    provider::Provider, signers::private_key::PrivateKeySigner, wallet::Wallet, Account,
    ViewOnlyAccount,
};
use portpicker::Port;
use rand::thread_rng;
use rexpect::spawn;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
    str::FromStr,
};
use tempfile::tempdir;
use toml_edit::{value, DocumentMut, InlineTable, Item, Table, Value};

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../")
        .join("../")
        .canonicalize()
        .unwrap()
}

fn test_data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test")
        .join("data")
        .canonicalize()
        .unwrap()
}

fn run_node() -> (Child, Port) {
    let port = portpicker::pick_unused_port().expect("No ports free");

    let child = Command::new("fuel-core")
        .arg("run")
        .arg("--debug")
        .arg("--db-type")
        .arg("in-memory")
        .arg("--port")
        .arg(port.to_string())
        .spawn()
        .expect("Failed to start fuel-core");
    (child, port)
}

/// Copy a directory recursively from `source` to `dest`.
fn copy_dir(source: &Path, dest: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dest)?;
    for e in fs::read_dir(source)? {
        let entry = e?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(&entry.path(), &dest.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dest.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Tries to get an `DeployedContract` out of the given `DeployedPackage`.
/// Panics otherwise.
fn expect_deployed_contract(deployed_package: DeployedPackage) -> DeployedContract {
    if let DeployedPackage::Contract(contract) = deployed_package {
        contract
    } else {
        println!("{deployed_package:?}");
        panic!("expected deployed package to be a contract")
    }
}

/// Tries to get a script (`DeployedExecutable`) out of given deployed package.
/// Panics otherwise.
fn expect_deployed_script(deployed_package: DeployedPackage) -> DeployedExecutable {
    if let DeployedPackage::Script(script) = deployed_package {
        script
    } else {
        panic!("expected deployed package to be a script")
    }
}

/// Tries to get a predicate (`DeployedExecutable`) out of given deployed package.
/// Panics otherwise.
fn expect_deployed_predicate(deployed_package: DeployedPackage) -> DeployedExecutable {
    if let DeployedPackage::Predicate(predicate) = deployed_package {
        predicate
    } else {
        panic!("expected deployed package to be a predicate")
    }
}

fn patch_manifest_file_with_path_std(manifest_dir: &Path) -> anyhow::Result<()> {
    let toml_path = manifest_dir.join(sway_utils::constants::MANIFEST_FILE_NAME);
    let toml_content = fs::read_to_string(&toml_path).unwrap();

    let mut doc = toml_content.parse::<DocumentMut>().unwrap();
    let new_std_path = get_workspace_root().join("sway-lib-std");

    let mut std_dependency = InlineTable::new();
    std_dependency.insert("path", Value::from(new_std_path.display().to_string()));
    doc["dependencies"]["std"] = Item::Value(Value::InlineTable(std_dependency));

    fs::write(&toml_path, doc.to_string()).unwrap();
    Ok(())
}

fn patch_manifest_file_with_proxy_table(manifest_dir: &Path, proxy: Proxy) -> anyhow::Result<()> {
    let toml_path = manifest_dir.join(sway_utils::constants::MANIFEST_FILE_NAME);
    let toml_content = fs::read_to_string(&toml_path)?;
    let mut doc = toml_content.parse::<DocumentMut>()?;

    let proxy_table = doc.entry("proxy").or_insert(Item::Table(Table::new()));
    let proxy_table = proxy_table.as_table_mut().unwrap();

    proxy_table.insert("enabled", value(proxy.enabled));

    if let Some(address) = proxy.address {
        proxy_table.insert("address", value(address));
    } else {
        proxy_table.remove("address");
    }

    fs::write(&toml_path, doc.to_string())?;
    Ok(())
}

fn update_main_sw(tmp_dir: &Path) -> anyhow::Result<()> {
    let main_sw_path = tmp_dir.join("src").join("main.sw");
    let content = fs::read_to_string(&main_sw_path)?;
    let updated_content = content.replace("true", "false");
    fs::write(main_sw_path, updated_content)?;
    Ok(())
}

async fn assert_big_contract_calls(wallet: Wallet, contract_id: ContractId) {
    abigen!(Contract(
        name = "BigContract",
        abi = "forc-plugins/forc-client/test/data/big_contract/big_contract-abi.json"
    ));

    let instance = BigContract::new(contract_id, wallet);

    let result = instance.methods().large_blob().call().await.unwrap().value;
    assert!(result);

    let result = instance
        .methods()
        .enum_input_output(Location::Mars)
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(result, Location::Mars);

    // Test enum with "tuple like struct" with simple value.
    let result = instance
        .methods()
        .enum_input_output(Location::Earth(u64::MAX))
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(result, Location::Earth(u64::MAX));

    // Test enum with "tuple like struct" with enum value.
    let result = instance
        .methods()
        .enum_input_output(Location::SimpleJupiter(Color::Red))
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(result, Location::SimpleJupiter(Color::Red));

    // Test enum with "tuple like struct" with enum value.
    let result = instance
        .methods()
        .enum_input_output(Location::SimpleJupiter(Color::Blue(u64::MAX)))
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(result, Location::SimpleJupiter(Color::Blue(u64::MAX)));

    // Test enum with "tuple like struct" with enum array value.
    let result = instance
        .methods()
        .enum_input_output(Location::Jupiter([Color::Red, Color::Blue(u64::MAX)]))
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(
        result,
        Location::Jupiter([Color::Red, Color::Blue(u64::MAX)])
    );

    // Test enum with "tuple like struct" with struct array value.
    let result = instance
        .methods()
        .enum_input_output(Location::SimplePluto(SimpleStruct {
            a: true,
            b: u64::MAX,
        }))
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(
        result,
        Location::SimplePluto(SimpleStruct {
            a: true,
            b: u64::MAX,
        })
    );

    let input = Person {
        name: AsciiString::new("Alice".into()).unwrap(),
        age: 42,
        alive: true,
        location: Location::Earth(1),
        some_tuple: (false, 42),
        some_array: [4, 2],
        some_b_256: Bits256::zeroed(),
    };
    let result = instance
        .methods()
        .struct_input_output(input.clone())
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(result, input);

    let _ = instance
        .methods()
        .push_storage_u16(42)
        .call()
        .await
        .unwrap();
    let result = instance
        .methods()
        .get_storage_u16(0)
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(result, 42);

    let _ = instance
        .methods()
        .push_storage_simple(SimpleStruct {
            a: true,
            b: u64::MAX,
        })
        .call()
        .await
        .unwrap();
    let result = instance
        .methods()
        .get_storage_simple(0)
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(
        result,
        SimpleStruct {
            a: true,
            b: u64::MAX,
        }
    );

    let _ = instance
        .methods()
        .push_storage_location(Location::Mars)
        .call()
        .await
        .unwrap();
    let result = instance
        .methods()
        .get_storage_location(0)
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(result, Location::Mars);

    let _ = instance
        .methods()
        .push_storage_location(Location::Earth(u64::MAX))
        .call()
        .await
        .unwrap();
    let result = instance
        .methods()
        .get_storage_location(1)
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(result, Location::Earth(u64::MAX));

    let _ = instance
        .methods()
        .push_storage_location(Location::Jupiter([Color::Red, Color::Blue(u64::MAX)]))
        .call()
        .await
        .unwrap();
    let result = instance
        .methods()
        .get_storage_location(2)
        .call()
        .await
        .unwrap()
        .value;
    assert_eq!(
        result,
        Location::Jupiter([Color::Red, Color::Blue(u64::MAX)])
    );

    let result = instance
        .methods()
        .assert_configurables()
        .call()
        .await
        .unwrap()
        .value;
    assert!(result);
}

#[tokio::test]
async fn test_simple_deploy() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("standalone_contract");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let contract_ids = deploy(cmd).await.unwrap();
    node.kill().unwrap();
    let expected = vec![DeployedPackage::Contract(DeployedContract {
        id: ContractId::from_str(
            "677a9eefe864cde328b1f6e58a0d9829fc3b683fca48e36e9bcb4863179ae174",
        )
        .unwrap(),
        proxy: None,
        chunked: false,
    })];

    assert_eq!(contract_ids, expected)
}

#[tokio::test]
async fn test_deploy_submit_only() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("standalone_contract");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");

    let target = NodeTarget {
        node_url: Some(node_url),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        submit_only: true,
        ..Default::default()
    };
    let contract_ids = deploy(cmd).await.unwrap();
    node.kill().unwrap();
    let expected = vec![DeployedPackage::Contract(DeployedContract {
        id: ContractId::from_str(
            "677a9eefe864cde328b1f6e58a0d9829fc3b683fca48e36e9bcb4863179ae174",
        )
        .unwrap(),
        proxy: None,
        chunked: false,
    })];

    assert_eq!(contract_ids, expected)
}

#[tokio::test]
async fn test_deploy_fresh_proxy() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("standalone_contract");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();
    let proxy = Proxy {
        enabled: true,
        address: None,
    };
    patch_manifest_file_with_proxy_table(tmp_dir.path(), proxy).unwrap();

    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let contract_ids = deploy(cmd).await.unwrap();
    node.kill().unwrap();
    let impl_contract = DeployedPackage::Contract(DeployedContract {
        id: ContractId::from_str(
            "677a9eefe864cde328b1f6e58a0d9829fc3b683fca48e36e9bcb4863179ae174",
        )
        .unwrap(),
        proxy: Some(
            ContractId::from_str(
                "fedbb732b17cf256aa378584438a154d11d413d5cfbdeca63a00128530aa0ebb",
            )
            .unwrap(),
        ),
        chunked: false,
    });
    let expected = vec![impl_contract];

    assert_eq!(contract_ids, expected)
}

#[tokio::test]
async fn test_proxy_contract_re_routes_call() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("standalone_contract");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();
    let proxy = Proxy {
        enabled: true,
        address: None,
    };
    patch_manifest_file_with_proxy_table(tmp_dir.path(), proxy).unwrap();

    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_contract = expect_deployed_contract(deploy(cmd).await.unwrap().remove(0));
    // At this point we deployed a contract with proxy.
    let proxy_contract_id = deployed_contract.proxy.unwrap();
    let impl_contract_id = deployed_contract.id;
    // Make a contract call into proxy contract, and check if the initial
    // contract returns a true.
    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let signer = PrivateKeySigner::new(secret_key);
    let wallet_unlocked = Wallet::new(signer, provider);

    abigen!(Contract(
        name = "ImplementationContract",
        abi = "forc-plugins/forc-client/test/data/standalone_contract/standalone_contract-abi.json"
    ));

    let impl_contract_a = ImplementationContract::new(proxy_contract_id, wallet_unlocked.clone());

    // Test storage functions
    let res = impl_contract_a
        .methods()
        .test_function_read()
        .with_contract_ids(&[impl_contract_id])
        .call()
        .await
        .unwrap();
    assert_eq!(res.value, 5);
    let res = impl_contract_a
        .methods()
        .test_function_write(8)
        .with_contract_ids(&[impl_contract_id])
        .call()
        .await
        .unwrap();
    assert_eq!(res.value, 8);

    let res = impl_contract_a
        .methods()
        .test_function()
        .with_contract_ids(&[impl_contract_id])
        .call()
        .await
        .unwrap();
    assert!(res.value);

    update_main_sw(tmp_dir.path()).unwrap();
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_contract = expect_deployed_contract(deploy(cmd).await.unwrap().remove(0));
    // proxy contract id should be the same.
    let proxy_contract_after_update = deployed_contract.proxy.unwrap();
    assert_eq!(proxy_contract_id, proxy_contract_after_update);
    let impl_contract_id_after_update = deployed_contract.id;
    assert!(impl_contract_id != impl_contract_id_after_update);
    let impl_contract_a = ImplementationContract::new(proxy_contract_after_update, wallet_unlocked);

    // Test storage functions
    let res = impl_contract_a
        .methods()
        .test_function_read()
        .with_contract_ids(&[impl_contract_id_after_update])
        .call()
        .await
        .unwrap();
    // Storage should be preserved from the previous target contract.
    assert_eq!(res.value, 8);
    let res = impl_contract_a
        .methods()
        .test_function_write(9)
        .with_contract_ids(&[impl_contract_id_after_update])
        .call()
        .await
        .unwrap();
    assert_eq!(res.value, 9);

    let res = impl_contract_a
        .methods()
        .test_function()
        .with_contract_ids(&[impl_contract_id_after_update])
        .call()
        .await
        .unwrap();
    assert!(!res.value);
    node.kill().unwrap();
}

#[tokio::test]
async fn test_non_owner_fails_to_set_target() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("standalone_contract");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();
    let proxy = Proxy {
        enabled: true,
        address: None,
    };
    patch_manifest_file_with_proxy_table(tmp_dir.path(), proxy).unwrap();

    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let contract_id = expect_deployed_contract(deploy(cmd).await.unwrap().remove(0));
    // Proxy contract's id.
    let proxy_id = contract_id.proxy.unwrap();

    // Create and fund an owner account and an attacker account.
    let provider = Provider::connect(&node_url).await.unwrap();
    let attacker_secret_key = SecretKey::random(&mut thread_rng());
    let attacker_signer = PrivateKeySigner::new(attacker_secret_key);
    let attacker_wallet = Wallet::new(attacker_signer.clone(), provider.clone());

    let owner_secret_key =
        SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let owner_signer = PrivateKeySigner::new(owner_secret_key);
    let owner_wallet = Wallet::new(owner_signer, provider.clone());
    let consensus_parameters = provider.consensus_parameters().await.unwrap();
    let base_asset_id = consensus_parameters.base_asset_id();

    // Fund attacker wallet so that it can try to make a set proxy target call.
    owner_wallet
        .transfer(
            attacker_wallet.address(),
            100000,
            *base_asset_id,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    let dummy_contract_id_target = ContractId::default();
    abigen!(Contract(name = "ProxyContract", abi = "{\"programType\":\"contract\",\"specVersion\":\"1.1\",\"encodingVersion\":\"1\",\"concreteTypes\":[{\"type\":\"()\",\"concreteTypeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"type\":\"enum standards::src5::AccessError\",\"concreteTypeId\":\"3f702ea3351c9c1ece2b84048006c8034a24cbc2bad2e740d0412b4172951d3d\",\"metadataTypeId\":1},{\"type\":\"enum standards::src5::State\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"metadataTypeId\":2},{\"type\":\"enum std::option::Option<struct std::contract_id::ContractId>\",\"concreteTypeId\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"metadataTypeId\":4,\"typeArguments\":[\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\"]},{\"type\":\"enum sway_libs::ownership::errors::InitializationError\",\"concreteTypeId\":\"1dfe7feadc1d9667a4351761230f948744068a090fe91b1bc6763a90ed5d3893\",\"metadataTypeId\":5},{\"type\":\"enum sway_libs::upgradability::errors::SetProxyOwnerError\",\"concreteTypeId\":\"3c6e90ae504df6aad8b34a93ba77dc62623e00b777eecacfa034a8ac6e890c74\",\"metadataTypeId\":6},{\"type\":\"str\",\"concreteTypeId\":\"8c25cb3686462e9a86d2883c5688a22fe738b0bbc85f458d2d2b5f3f667c6d5a\"},{\"type\":\"struct std::contract_id::ContractId\",\"concreteTypeId\":\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\",\"metadataTypeId\":9},{\"type\":\"struct sway_libs::upgradability::events::ProxyOwnerSet\",\"concreteTypeId\":\"96dd838b44f99d8ccae2a7948137ab6256c48ca4abc6168abc880de07fba7247\",\"metadataTypeId\":10},{\"type\":\"struct sway_libs::upgradability::events::ProxyTargetSet\",\"concreteTypeId\":\"1ddc0adda1270a016c08ffd614f29f599b4725407c8954c8b960bdf651a9a6c8\",\"metadataTypeId\":11}],\"metadataTypes\":[{\"type\":\"b256\",\"metadataTypeId\":0},{\"type\":\"enum standards::src5::AccessError\",\"metadataTypeId\":1,\"components\":[{\"name\":\"NotOwner\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum standards::src5::State\",\"metadataTypeId\":2,\"components\":[{\"name\":\"Uninitialized\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"name\":\"Initialized\",\"typeId\":3},{\"name\":\"Revoked\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum std::identity::Identity\",\"metadataTypeId\":3,\"components\":[{\"name\":\"Address\",\"typeId\":8},{\"name\":\"ContractId\",\"typeId\":9}]},{\"type\":\"enum std::option::Option\",\"metadataTypeId\":4,\"components\":[{\"name\":\"None\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"name\":\"Some\",\"typeId\":7}],\"typeParameters\":[7]},{\"type\":\"enum sway_libs::ownership::errors::InitializationError\",\"metadataTypeId\":5,\"components\":[{\"name\":\"CannotReinitialized\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum sway_libs::upgradability::errors::SetProxyOwnerError\",\"metadataTypeId\":6,\"components\":[{\"name\":\"CannotUninitialize\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"generic T\",\"metadataTypeId\":7},{\"type\":\"struct std::address::Address\",\"metadataTypeId\":8,\"components\":[{\"name\":\"bits\",\"typeId\":0}]},{\"type\":\"struct std::contract_id::ContractId\",\"metadataTypeId\":9,\"components\":[{\"name\":\"bits\",\"typeId\":0}]},{\"type\":\"struct sway_libs::upgradability::events::ProxyOwnerSet\",\"metadataTypeId\":10,\"components\":[{\"name\":\"new_proxy_owner\",\"typeId\":2}]},{\"type\":\"struct sway_libs::upgradability::events::ProxyTargetSet\",\"metadataTypeId\":11,\"components\":[{\"name\":\"new_target\",\"typeId\":9}]}],\"functions\":[{\"inputs\":[],\"name\":\"proxy_target\",\"output\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Returns the target contract of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Returns\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * [Option<ContractId>] - The new proxy contract to which all fallback calls will be passed or `None`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\"]}]},{\"inputs\":[{\"name\":\"new_target\",\"concreteTypeId\":\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\"}],\"name\":\"set_proxy_target\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Change the target contract of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can only be called by the `proxy_owner`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Arguments\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * `new_target`: [ContractId] - The new proxy contract to which all fallback calls will be passed.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When not called by `proxy_owner`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Write: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\",\"write\"]}]},{\"inputs\":[],\"name\":\"proxy_owner\",\"output\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Returns the owner of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Returns\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * [State] - Represents the state of ownership for this contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\"]}]},{\"inputs\":[],\"name\":\"initialize_proxy\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Initializes the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method sets the storage values using the values of the configurable constants `INITIAL_TARGET` and `INITIAL_OWNER`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This then allows methods that write to storage to be called.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can only be called once.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When `storage::SRC14.proxy_owner` is not [State::Uninitialized].\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Writes: `2`\"]},{\"name\":\"storage\",\"arguments\":[\"write\"]}]},{\"inputs\":[{\"name\":\"new_proxy_owner\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\"}],\"name\":\"set_proxy_owner\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Changes proxy ownership to the passed State.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can be used to transfer ownership between Identities or to revoke ownership.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Arguments\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * `new_proxy_owner`: [State] - The new state of the proxy ownership.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When the sender is not the current proxy owner.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When the new state of the proxy ownership is [State::Uninitialized].\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Writes: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"write\"]}]}],\"loggedTypes\":[{\"logId\":\"4571204900286667806\",\"concreteTypeId\":\"3f702ea3351c9c1ece2b84048006c8034a24cbc2bad2e740d0412b4172951d3d\"},{\"logId\":\"2151606668983994881\",\"concreteTypeId\":\"1ddc0adda1270a016c08ffd614f29f599b4725407c8954c8b960bdf651a9a6c8\"},{\"logId\":\"2161305517876418151\",\"concreteTypeId\":\"1dfe7feadc1d9667a4351761230f948744068a090fe91b1bc6763a90ed5d3893\"},{\"logId\":\"4354576968059844266\",\"concreteTypeId\":\"3c6e90ae504df6aad8b34a93ba77dc62623e00b777eecacfa034a8ac6e890c74\"},{\"logId\":\"10870989709723147660\",\"concreteTypeId\":\"96dd838b44f99d8ccae2a7948137ab6256c48ca4abc6168abc880de07fba7247\"},{\"logId\":\"10098701174489624218\",\"concreteTypeId\":\"8c25cb3686462e9a86d2883c5688a22fe738b0bbc85f458d2d2b5f3f667c6d5a\"}],\"messagesTypes\":[],\"configurables\":[{\"name\":\"INITIAL_TARGET\",\"concreteTypeId\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"offset\":13368},{\"name\":\"INITIAL_OWNER\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"offset\":13320}]}",));

    let wallet = Wallet::new(attacker_signer, provider.clone());
    let attacker_account = ForcClientAccount::Wallet(wallet);
    // Try to change target of the proxy with a random wallet which is not the owner of the proxy.
    let res = update_proxy_contract_target(&attacker_account, proxy_id, dummy_contract_id_target)
        .await
        .err()
        .unwrap();

    node.kill().unwrap();
    assert!(res
        .to_string()
        .starts_with("transaction reverted: NotOwner"));
}

// TODO: https://github.com/FuelLabs/sway/issues/6283
// Add interactive tests for the happy path cases. This requires starting the node with funded accounts and setting up
// the wallet with the correct password. The tests should be run in a separate test suite that is not run by default.
// It would also require overriding `default_wallet_path` function for tests, so as not to interfere with the user's wallet.

#[test]
fn test_deploy_interactive_missing_wallet() -> Result<(), rexpect::error::Error> {
    let (mut node, port) = run_node();
    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");

    // Spawn the forc-deploy binary using cargo run
    let project_dir = test_data_path().join("standalone_contract");
    let mut process = spawn(
        &format!(
            "cargo run --bin forc-deploy -- --node-url {node_url} -p {}",
            project_dir.display()
        ),
        Some(300000),
    )?;

    // Confirmation prompts
    process
        .exp_string("\u{1b}[1;32mConfirming\u{1b}[0m transactions [deploy standalone_contract]")?;
    process.exp_string(&format!("Network: {node_url}"))?;
    process.exp_regex("Could not find a wallet at")?;
    process.send_line("n")?;

    process.process.exit()?;
    node.kill().unwrap();
    Ok(())
}

#[tokio::test]
async fn chunked_deploy() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("big_contract");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_contract = expect_deployed_contract(deploy(cmd).await.unwrap().remove(0));
    node.kill().unwrap();

    assert!(deployed_contract.chunked);
}

#[tokio::test]
async fn chunked_deploy_re_routes_calls() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("big_contract");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_contract = expect_deployed_contract(deploy(cmd).await.unwrap().remove(0));

    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let signer = PrivateKeySigner::new(secret_key);
    let wallet_unlocked = Wallet::new(signer, provider);

    assert_big_contract_calls(wallet_unlocked, deployed_contract.id).await;

    node.kill().unwrap();
}

#[tokio::test]
async fn chunked_deploy_with_proxy_re_routes_call() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("big_contract");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();
    let proxy = Proxy {
        enabled: true,
        address: None,
    };
    patch_manifest_file_with_proxy_table(tmp_dir.path(), proxy).unwrap();

    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_contract = expect_deployed_contract(deploy(cmd).await.unwrap().remove(0));

    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let signer = PrivateKeySigner::new(secret_key);
    let wallet_unlocked = Wallet::new(signer, provider);

    assert_big_contract_calls(wallet_unlocked, deployed_contract.id).await;

    node.kill().unwrap();
}

#[tokio::test]
async fn can_deploy_script() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("deployed_script");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };

    expect_deployed_script(deploy(cmd).await.unwrap().remove(0));
    node.kill().unwrap();
}

#[tokio::test]
async fn deploy_script_calls() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("deployed_script");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };

    expect_deployed_script(deploy(cmd).await.unwrap().remove(0));

    // Deploy the contract the script is going to be calling.
    let contract_tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("standalone_contract");
    copy_dir(&project_dir, contract_tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(contract_tmp_dir.path()).unwrap();

    let pkg = Pkg {
        path: Some(contract_tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_packages = deploy(cmd).await.unwrap().remove(0);
    let contract = expect_deployed_contract(deployed_packages);
    let contract_id = contract.id;

    abigen!(Script(
        name = "MyScript",
        abi = "forc-plugins/forc-client/test/data/deployed_script/deployed_script-abi.json"
    ));

    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let signer = PrivateKeySigner::new(secret_key);
    let wallet_unlocked = Wallet::new(signer, provider);

    let loader_path = tmp_dir.path().join("out/deployed_script-loader.bin");
    let instance = MyScript::new(wallet_unlocked, &loader_path.display().to_string());

    let contract_id_bits256 = Bits256(contract.id.into());
    let call_handler = instance
        .main(10, contract_id_bits256)
        .with_contract_ids(&[contract_id])
        .call()
        .await
        .unwrap();
    let (configs, with_input, without_input, from_contract) = call_handler.value;
    let receipts = call_handler.tx_status.receipts;

    assert!(configs.0); // bool
    assert_eq!(configs.1, 8); // u8
    assert_eq!(configs.2, 16); // u16
    assert_eq!(configs.3, 32); // u32
    assert_eq!(configs.4, 63); // u64
    assert_eq!(configs.5, 8.into()); // u256
    assert_eq!(
        configs.6,
        Bits256::from_hex_str("0x0101010101010101010101010101010101010101010101010101010101010101")
            .unwrap()
    ); // b256
    assert_eq!(
        configs.7,
        SizedAsciiString::new("fuel".to_string()).unwrap()
    ); // str[4]
    assert_eq!(configs.8, (8, true)); // tuple
    assert_eq!(configs.9, [253, 254, 255]); // array

    let expected_struct = StructWithGeneric {
        field_1: 8,
        field_2: 16,
    };
    assert_eq!(configs.10, expected_struct); // struct

    let expected_enum = EnumWithGeneric::VariantOne(true);
    assert_eq!(configs.11, expected_enum); // enum

    assert!(with_input); // 10 % 2 == 0
    assert_eq!(without_input, 2500); // 25 * 100 = 2500

    assert_eq!(from_contract, 5);

    receipts.iter().find(|receipt| {
        if let fuel_tx::Receipt::LogData { data, .. } = receipt {
            matches!(data.as_ref(), Some(bytes) if bytes.as_ref() == [0x08])
        } else {
            false
        }
    });

    node.kill().unwrap();
}

#[tokio::test]
async fn can_deploy_predicates() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("deployed_predicate");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };

    expect_deployed_predicate(deploy(cmd).await.unwrap().remove(0));
    node.kill().unwrap();
}

#[tokio::test]
async fn deployed_predicate_call() {
    let (mut node, port) = run_node();
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("deployed_predicate");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    expect_deployed_predicate(deploy(cmd).await.unwrap().remove(0));

    abigen!(Predicate(
        name = "MyPredicate",
        abi = "forc-plugins/forc-client/test/data/deployed_predicate/deployed_predicate-abi.json"
    ));

    let provider = Provider::connect(&node_url).await.unwrap();
    let consensus_parameters = provider.consensus_parameters().await.unwrap();
    let base_asset_id = consensus_parameters.base_asset_id();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let signer = PrivateKeySigner::new(secret_key);
    let wallet_unlocked = Wallet::new(signer, provider.clone());
    let loader_path = tmp_dir.path().join("out/deployed_predicate-loader.bin");
    let strct = StructWithGeneric {
        field_1: 8,
        field_2: 16,
    };
    let enm = EnumWithGeneric::VariantOne(true);
    let encoded_data = MyPredicateEncoder::default()
        .encode_data(true, 8, strct, enm)
        .unwrap();
    let predicate: fuels::prelude::Predicate =
        fuels::prelude::Predicate::load_from(&loader_path.display().to_string())
            .unwrap()
            .with_data(encoded_data)
            .with_provider(provider);

    // lock some amount under the predicate
    wallet_unlocked
        .transfer(
            predicate.address(),
            5000,
            *base_asset_id,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // Check predicate balance.
    let balance_before = predicate.get_asset_balance(base_asset_id).await.unwrap();
    assert_eq!(balance_before, 5000);

    // Try to spend it
    let amount_to_unlock = 300;
    let response = predicate
        .transfer(
            wallet_unlocked.address(),
            amount_to_unlock,
            *base_asset_id,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // Check predicate balance again.
    let balance_after = predicate.get_asset_balance(base_asset_id).await.unwrap();
    let total_fee = response.tx_status.total_fee as u128;
    let spent = balance_before
        .checked_sub(balance_after)
        .expect("predicate balance increased unexpectedly");
    // Provider::transfer currently over-estimates the required fee in some cases, so we check
    // the actual spend from the predicate instead of trusting the reported fee blindly.
    let actual_fee = spent
        .checked_sub(amount_to_unlock as u128)
        .expect("predicate spent less than the unlocked amount");
    assert!(
        actual_fee <= total_fee,
        "network fee {actual_fee} exceeded reported total_fee {total_fee}"
    );

    node.kill().unwrap();
}

/// Generates a script instance using SDK, and returns the result as a string.
async fn call_with_sdk_generated_overrides(node_url: &str, contract_id: ContractId) -> String {
    let project_dir = test_data_path().join("deployed_script");
    abigen!(Script(
        name = "MyScript",
        abi = "forc-plugins/forc-client/test/data/deployed_script/deployed_script-abi.json"
    ));
    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let signer = PrivateKeySigner::new(secret_key);
    let wallet_unlocked = Wallet::new(signer, provider);
    let bin_dir = project_dir.join("deployed_script.bin");
    let script_instance = MyScript::new(wallet_unlocked, bin_dir.display().to_string().as_str());

    let strc = StructWithGeneric {
        field_1: 1u8,
        field_2: 2,
    };
    let encoded = MyScriptConfigurables::default()
        .with_BOOL(false)
        .unwrap()
        .with_U8(1)
        .unwrap()
        .with_U16(2)
        .unwrap()
        .with_U32(3)
        .unwrap()
        .with_U64(4)
        .unwrap()
        .with_U256(5.into())
        .unwrap()
        .with_B256(Bits256::zeroed())
        .unwrap()
        .with_ARRAY([1, 2, 3])
        .unwrap()
        .with_STRUCT(strc)
        .unwrap()
        .with_ENUM(EnumWithGeneric::VariantTwo)
        .unwrap();

    let mut script_instance_with_configs = script_instance.with_configurables(encoded);

    let loader_from_sdk = script_instance_with_configs
        .convert_into_loader()
        .await
        .unwrap();

    let contract_ids_bits256 = Bits256(contract_id.into());
    format!(
        "{:?}",
        loader_from_sdk
            .main(10, contract_ids_bits256)
            .with_contract_ids(&[contract_id])
            .call()
            .await
            .unwrap()
            .value
    )
}

/// Generates a script instance using the shifted abi, and returns the result as a string.
async fn call_with_forc_generated_overrides(node_url: &str, contract_id: ContractId) -> String {
    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let signer = PrivateKeySigner::new(secret_key);
    let wallet_unlocked = Wallet::new(signer, provider);
    let tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("deployed_script");
    copy_dir(&project_dir, tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(tmp_dir.path()).unwrap();

    let target = NodeTarget {
        node_url: Some(node_url.to_string()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let pkg = Pkg {
        path: Some(tmp_dir.path().display().to_string()),
        ..Default::default()
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };

    expect_deployed_script(deploy(cmd).await.unwrap().remove(0));

    // Since `abigen!` macro does not allow for dynamic paths, we need to
    // pre-generate the loader bin and abi and read them from project dir. Here
    // we are ensuring forc-deploy indeed generated the files we are basing our
    // tests below.
    let generated_loader_abi_path = tmp_dir.path().join("out/deployed_script-loader-abi.json");
    let generated_loader_abi = fs::read_to_string(generated_loader_abi_path).unwrap();

    // this path is basically, `forc-plugins/forc-client/test/data/deployed_script/deployed_script-loader-abi.json`.
    let used_loader_abi_path = project_dir.join("deployed_script-loader-abi.json");
    let used_loader_abi = fs::read_to_string(used_loader_abi_path).unwrap();

    assert_eq!(generated_loader_abi, used_loader_abi);

    let generated_loader_bin = tmp_dir.path().join("out/deployed_script-loader.bin");
    abigen!(Script(
        name = "MyScript",
        abi = "forc-plugins/forc-client/test/data/deployed_script/deployed_script-loader-abi.json"
    ));
    let forc_generated_script_instance = MyScript::new(
        wallet_unlocked,
        generated_loader_bin.display().to_string().as_str(),
    );
    let strc = StructWithGeneric {
        field_1: 1u8,
        field_2: 2,
    };
    let encoded = MyScriptConfigurables::default()
        .with_BOOL(false)
        .unwrap()
        .with_U8(1)
        .unwrap()
        .with_U16(2)
        .unwrap()
        .with_U32(3)
        .unwrap()
        .with_U64(4)
        .unwrap()
        .with_U256(5.into())
        .unwrap()
        .with_B256(Bits256::zeroed())
        .unwrap()
        .with_ARRAY([1, 2, 3])
        .unwrap()
        .with_STRUCT(strc)
        .unwrap()
        .with_ENUM(EnumWithGeneric::VariantTwo)
        .unwrap();

    let forc_generated_script_with_configs =
        forc_generated_script_instance.with_configurables(encoded);
    let contract_ids_bits256 = Bits256(contract_id.into());
    format!(
        "{:?}",
        forc_generated_script_with_configs
            .main(10, contract_ids_bits256)
            .with_contract_ids(&[contract_id])
            .call()
            .await
            .unwrap()
            .value
    )
}

#[tokio::test]
async fn offset_shifted_abi_works() {
    // To test if offset shifted abi works or not, we generate a loader
    // contract using sdk and give a configurable override, and call the
    // main function.

    // We also create the shifted abi using forc-deploy and create a script
    // instance using this new shifted abi, and generate a normal script out of
    // the loader binary generated again by forc-deploy.

    // We then override the configurables with the same values as sdk flow on
    // this script, generated with loader abi and bin coming from forc-deploy.

    // If returned value is equal, than the configurables work correctly.
    let (mut node, port) = run_node();
    // Deploy the contract the script is going to be calling.
    let contract_tmp_dir = tempdir().unwrap();
    let project_dir = test_data_path().join("standalone_contract");
    copy_dir(&project_dir, contract_tmp_dir.path()).unwrap();
    patch_manifest_file_with_path_std(contract_tmp_dir.path()).unwrap();

    let pkg = Pkg {
        path: Some(contract_tmp_dir.path().display().to_string()),
        ..Default::default()
    };

    let node_url = format!("http://127.0.0.1:{port}/v1/graphql");
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
        mainnet: false,
        devnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_packages = deploy(cmd).await.unwrap().remove(0);
    let contract = expect_deployed_contract(deployed_packages);
    let contract_id = contract.id;
    // Generating the sdk loader bytecode with configurables.
    let loader_with_configs_from_sdk =
        call_with_sdk_generated_overrides(&node_url, contract_id).await;

    // Generating the forc-deploy loader bytecode and loader abi.
    let loader_with_configs_from_forc =
        call_with_forc_generated_overrides(&node_url, contract_id).await;
    pretty_assertions::assert_eq!(loader_with_configs_from_forc, loader_with_configs_from_sdk);

    node.kill().unwrap()
}
