use forc::cli::shared::Pkg;
use forc_client::{
    cmd,
    op::{deploy, DeployedContract},
    util::{account::ForcClientAccount, tx::update_proxy_contract_target},
    NodeTarget,
};
use forc_pkg::manifest::Proxy;
use fuel_crypto::SecretKey;
use fuel_tx::{ContractId, Salt};
use fuels::{
    macros::abigen,
    types::{transaction::TxPolicies, AsciiString, Bits256},
};
use fuels_accounts::{provider::Provider, wallet::WalletUnlocked, Account};
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

async fn assert_big_contract_calls(wallet: WalletUnlocked, contract_id: ContractId) {
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

    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);
    let target = NodeTarget {
        node_url: Some(node_url),
        target: None,
        testnet: false,
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
    let expected = vec![DeployedContract {
        id: ContractId::from_str(
            "4ea5fa100cd7c8972bc8925ed6f8ccfb6bf1e16f79c3642c3a503c73b7d18de2",
        )
        .unwrap(),
        proxy: None,
        chunked: false,
    }];

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

    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);

    let target = NodeTarget {
        node_url: Some(node_url),
        target: None,
        testnet: false,
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
    let expected = vec![DeployedContract {
        id: ContractId::from_str(
            "4ea5fa100cd7c8972bc8925ed6f8ccfb6bf1e16f79c3642c3a503c73b7d18de2",
        )
        .unwrap(),
        proxy: None,
        chunked: false,
    }];

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

    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);
    let target = NodeTarget {
        node_url: Some(node_url),
        target: None,
        testnet: false,
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
    let impl_contract = DeployedContract {
        id: ContractId::from_str(
            "4ea5fa100cd7c8972bc8925ed6f8ccfb6bf1e16f79c3642c3a503c73b7d18de2",
        )
        .unwrap(),
        proxy: Some(
            ContractId::from_str(
                "deb633128bceadcd4eb4fe546089f6653727348b60228638a7f9d55d0b6da1ae",
            )
            .unwrap(),
        ),
        chunked: false,
    };
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

    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let contract_ids = deploy(cmd).await.unwrap();
    // At this point we deployed a contract with proxy.
    let proxy_contract_id = contract_ids[0].proxy.unwrap();
    let impl_contract_id = contract_ids[0].id;
    // Make a contract call into proxy contract, and check if the initial
    // contract returns a true.
    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let wallet_unlocked = WalletUnlocked::new_from_private_key(secret_key, Some(provider));

    abigen!(Contract(
        name = "ImplementationContract",
        abi = "forc-plugins/forc-client/test/data/standalone_contract/standalone_contract-abi.json"
    ));

    let impl_contract_a = ImplementationContract::new(proxy_contract_id, wallet_unlocked.clone());

    // Test storage functions
    let res = impl_contract_a
        .methods()
        .test_function_read()
        .with_contract_ids(&[impl_contract_id.into()])
        .call()
        .await
        .unwrap();
    assert_eq!(res.value, 5);
    let res = impl_contract_a
        .methods()
        .test_function_write(8)
        .with_contract_ids(&[impl_contract_id.into()])
        .call()
        .await
        .unwrap();
    assert_eq!(res.value, 8);

    let res = impl_contract_a
        .methods()
        .test_function()
        .with_contract_ids(&[impl_contract_id.into()])
        .call()
        .await
        .unwrap();
    assert!(res.value);

    update_main_sw(tmp_dir.path()).unwrap();
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
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
    let contract_ids = deploy(cmd).await.unwrap();
    // proxy contract id should be the same.
    let proxy_contract_after_update = contract_ids[0].proxy.unwrap();
    assert_eq!(proxy_contract_id, proxy_contract_after_update);
    let impl_contract_id_after_update = contract_ids[0].id;
    assert!(impl_contract_id != impl_contract_id_after_update);
    let impl_contract_a = ImplementationContract::new(proxy_contract_after_update, wallet_unlocked);

    // Test storage functions
    let res = impl_contract_a
        .methods()
        .test_function_read()
        .with_contract_ids(&[impl_contract_id_after_update.into()])
        .call()
        .await
        .unwrap();
    // Storage should be preserved from the previous target contract.
    assert_eq!(res.value, 8);
    let res = impl_contract_a
        .methods()
        .test_function_write(9)
        .with_contract_ids(&[impl_contract_id_after_update.into()])
        .call()
        .await
        .unwrap();
    assert_eq!(res.value, 9);

    let res = impl_contract_a
        .methods()
        .test_function()
        .with_contract_ids(&[impl_contract_id_after_update.into()])
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

    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let contract_id = deploy(cmd).await.unwrap();
    // Proxy contract's id.
    let proxy_id = contract_id.first().and_then(|f| f.proxy).unwrap();

    // Create and fund an owner account and an attacker account.
    let provider = Provider::connect(&node_url).await.unwrap();
    let attacker_secret_key = SecretKey::random(&mut thread_rng());
    let attacker_wallet =
        WalletUnlocked::new_from_private_key(attacker_secret_key, Some(provider.clone()));

    let owner_secret_key =
        SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let owner_wallet =
        WalletUnlocked::new_from_private_key(owner_secret_key, Some(provider.clone()));
    let base_asset_id = provider.base_asset_id();

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
    abigen!(Contract(name = "ProxyContract", abi = "{\"programType\":\"contract\",\"specVersion\":\"1\",\"encodingVersion\":\"1\",\"concreteTypes\":[{\"type\":\"()\",\"concreteTypeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"type\":\"enum standards::src5::AccessError\",\"concreteTypeId\":\"3f702ea3351c9c1ece2b84048006c8034a24cbc2bad2e740d0412b4172951d3d\",\"metadataTypeId\":1},{\"type\":\"enum standards::src5::State\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"metadataTypeId\":2},{\"type\":\"enum std::option::Option<struct std::contract_id::ContractId>\",\"concreteTypeId\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"metadataTypeId\":4,\"typeArguments\":[\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\"]},{\"type\":\"enum sway_libs::ownership::errors::InitializationError\",\"concreteTypeId\":\"1dfe7feadc1d9667a4351761230f948744068a090fe91b1bc6763a90ed5d3893\",\"metadataTypeId\":5},{\"type\":\"enum sway_libs::upgradability::errors::SetProxyOwnerError\",\"concreteTypeId\":\"3c6e90ae504df6aad8b34a93ba77dc62623e00b777eecacfa034a8ac6e890c74\",\"metadataTypeId\":6},{\"type\":\"str\",\"concreteTypeId\":\"8c25cb3686462e9a86d2883c5688a22fe738b0bbc85f458d2d2b5f3f667c6d5a\"},{\"type\":\"struct std::contract_id::ContractId\",\"concreteTypeId\":\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\",\"metadataTypeId\":9},{\"type\":\"struct sway_libs::upgradability::events::ProxyOwnerSet\",\"concreteTypeId\":\"96dd838b44f99d8ccae2a7948137ab6256c48ca4abc6168abc880de07fba7247\",\"metadataTypeId\":10},{\"type\":\"struct sway_libs::upgradability::events::ProxyTargetSet\",\"concreteTypeId\":\"1ddc0adda1270a016c08ffd614f29f599b4725407c8954c8b960bdf651a9a6c8\",\"metadataTypeId\":11}],\"metadataTypes\":[{\"type\":\"b256\",\"metadataTypeId\":0},{\"type\":\"enum standards::src5::AccessError\",\"metadataTypeId\":1,\"components\":[{\"name\":\"NotOwner\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum standards::src5::State\",\"metadataTypeId\":2,\"components\":[{\"name\":\"Uninitialized\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"name\":\"Initialized\",\"typeId\":3},{\"name\":\"Revoked\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum std::identity::Identity\",\"metadataTypeId\":3,\"components\":[{\"name\":\"Address\",\"typeId\":8},{\"name\":\"ContractId\",\"typeId\":9}]},{\"type\":\"enum std::option::Option\",\"metadataTypeId\":4,\"components\":[{\"name\":\"None\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"},{\"name\":\"Some\",\"typeId\":7}],\"typeParameters\":[7]},{\"type\":\"enum sway_libs::ownership::errors::InitializationError\",\"metadataTypeId\":5,\"components\":[{\"name\":\"CannotReinitialized\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"enum sway_libs::upgradability::errors::SetProxyOwnerError\",\"metadataTypeId\":6,\"components\":[{\"name\":\"CannotUninitialize\",\"typeId\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\"}]},{\"type\":\"generic T\",\"metadataTypeId\":7},{\"type\":\"struct std::address::Address\",\"metadataTypeId\":8,\"components\":[{\"name\":\"bits\",\"typeId\":0}]},{\"type\":\"struct std::contract_id::ContractId\",\"metadataTypeId\":9,\"components\":[{\"name\":\"bits\",\"typeId\":0}]},{\"type\":\"struct sway_libs::upgradability::events::ProxyOwnerSet\",\"metadataTypeId\":10,\"components\":[{\"name\":\"new_proxy_owner\",\"typeId\":2}]},{\"type\":\"struct sway_libs::upgradability::events::ProxyTargetSet\",\"metadataTypeId\":11,\"components\":[{\"name\":\"new_target\",\"typeId\":9}]}],\"functions\":[{\"inputs\":[],\"name\":\"proxy_target\",\"output\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Returns the target contract of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Returns\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * [Option<ContractId>] - The new proxy contract to which all fallback calls will be passed or `None`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\"]}]},{\"inputs\":[{\"name\":\"new_target\",\"concreteTypeId\":\"29c10735d33b5159f0c71ee1dbd17b36a3e69e41f00fab0d42e1bd9f428d8a54\"}],\"name\":\"set_proxy_target\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Change the target contract of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can only be called by the `proxy_owner`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Arguments\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * `new_target`: [ContractId] - The new proxy contract to which all fallback calls will be passed.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When not called by `proxy_owner`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Write: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\",\"write\"]}]},{\"inputs\":[],\"name\":\"proxy_owner\",\"output\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Returns the owner of the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Returns\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * [State] - Represents the state of ownership for this contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"read\"]}]},{\"inputs\":[],\"name\":\"initialize_proxy\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Initializes the proxy contract.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method sets the storage values using the values of the configurable constants `INITIAL_TARGET` and `INITIAL_OWNER`.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This then allows methods that write to storage to be called.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can only be called once.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When `storage::SRC14.proxy_owner` is not [State::Uninitialized].\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Writes: `2`\"]},{\"name\":\"storage\",\"arguments\":[\"write\"]}]},{\"inputs\":[{\"name\":\"new_proxy_owner\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\"}],\"name\":\"set_proxy_owner\",\"output\":\"2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d\",\"attributes\":[{\"name\":\"doc-comment\",\"arguments\":[\" Changes proxy ownership to the passed State.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Additional Information\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" This method can be used to transfer ownership between Identities or to revoke ownership.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Arguments\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * `new_proxy_owner`: [State] - The new state of the proxy ownership.\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Reverts\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When the sender is not the current proxy owner.\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * When the new state of the proxy ownership is [State::Uninitialized].\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" # Number of Storage Accesses\"]},{\"name\":\"doc-comment\",\"arguments\":[\"\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Reads: `1`\"]},{\"name\":\"doc-comment\",\"arguments\":[\" * Writes: `1`\"]},{\"name\":\"storage\",\"arguments\":[\"write\"]}]}],\"loggedTypes\":[{\"logId\":\"4571204900286667806\",\"concreteTypeId\":\"3f702ea3351c9c1ece2b84048006c8034a24cbc2bad2e740d0412b4172951d3d\"},{\"logId\":\"2151606668983994881\",\"concreteTypeId\":\"1ddc0adda1270a016c08ffd614f29f599b4725407c8954c8b960bdf651a9a6c8\"},{\"logId\":\"2161305517876418151\",\"concreteTypeId\":\"1dfe7feadc1d9667a4351761230f948744068a090fe91b1bc6763a90ed5d3893\"},{\"logId\":\"4354576968059844266\",\"concreteTypeId\":\"3c6e90ae504df6aad8b34a93ba77dc62623e00b777eecacfa034a8ac6e890c74\"},{\"logId\":\"10870989709723147660\",\"concreteTypeId\":\"96dd838b44f99d8ccae2a7948137ab6256c48ca4abc6168abc880de07fba7247\"},{\"logId\":\"10098701174489624218\",\"concreteTypeId\":\"8c25cb3686462e9a86d2883c5688a22fe738b0bbc85f458d2d2b5f3f667c6d5a\"}],\"messagesTypes\":[],\"configurables\":[{\"name\":\"INITIAL_TARGET\",\"concreteTypeId\":\"0d79387ad3bacdc3b7aad9da3a96f4ce60d9a1b6002df254069ad95a3931d5c8\",\"offset\":13368},{\"name\":\"INITIAL_OWNER\",\"concreteTypeId\":\"192bc7098e2fe60635a9918afb563e4e5419d386da2bdbf0d716b4bc8549802c\",\"offset\":13320}]}",));

    let wallet = WalletUnlocked::new_from_private_key(attacker_secret_key, Some(provider.clone()));
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
fn test_deploy_interactive_wrong_password() -> Result<(), rexpect::error::Error> {
    let (mut node, port) = run_node();
    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);

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
    process.exp_string("Wallet: ")?;
    process.exp_string("Wallet password")?;
    process.send_line("mock_password")?;

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

    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);
    let target = NodeTarget {
        node_url: Some(node_url),
        target: None,
        testnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_contract = deploy(cmd).await.unwrap().remove(0);
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

    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_contract = deploy(cmd).await.unwrap().remove(0);

    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let wallet_unlocked = WalletUnlocked::new_from_private_key(secret_key, Some(provider));

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

    let node_url = format!("http://127.0.0.1:{}/v1/graphql", port);
    let target = NodeTarget {
        node_url: Some(node_url.clone()),
        target: None,
        testnet: false,
    };
    let cmd = cmd::Deploy {
        pkg,
        salt: Some(vec![format!("{}", Salt::default())]),
        node: target,
        default_signer: true,
        ..Default::default()
    };
    let deployed_contract = deploy(cmd).await.unwrap().remove(0);

    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let wallet_unlocked = WalletUnlocked::new_from_private_key(secret_key, Some(provider));

    assert_big_contract_calls(wallet_unlocked, deployed_contract.id).await;

    node.kill().unwrap();
}
