use forc::cli::shared::Pkg;
use forc_client::{
    cmd,
    op::{deploy, DeployedContract},
    util::tx::update_proxy_contract_target,
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
use toml_edit::{value, Document, InlineTable, Item, Table, Value};

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

    let mut doc = toml_content.parse::<Document>().unwrap();
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
    let mut doc = toml_content.parse::<Document>()?;

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
            "ad0bba17e0838ef859abe2693d8a5e3bc4e7cfb901601e30f4dc34999fda6335",
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
            "ad0bba17e0838ef859abe2693d8a5e3bc4e7cfb901601e30f4dc34999fda6335",
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
            "ad0bba17e0838ef859abe2693d8a5e3bc4e7cfb901601e30f4dc34999fda6335",
        )
        .unwrap(),
        proxy: Some(
            ContractId::from_str(
                "f2d67efbd6038c85ddaffdcdc859770d8bd20eeec8e3909911f1446b2ec9f764",
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
    abigen!(Contract(
        name = "ProxyContract",
        abi = "forc-plugins/forc-client/proxy_abi/proxy_contract-abi.json"
    ));

    // Try to change target of the proxy with a random wallet which is not the owner of the proxy.
    let res = update_proxy_contract_target(
        &provider,
        attacker_secret_key,
        proxy_id,
        dummy_contract_id_target,
    )
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
