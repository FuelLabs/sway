use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
    str::FromStr,
};

use forc::cli::shared::Pkg;
use forc_client::{
    cmd,
    op::{deploy, DeployedContract},
    NodeTarget,
};
use forc_pkg::manifest::Proxy;
use fuel_crypto::SecretKey;
use fuel_tx::{ContractId, Salt};
use fuels::macros::abigen;
use fuels_accounts::{provider::Provider, wallet::WalletUnlocked};
use portpicker::Port;
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

#[tokio::test]
async fn simple_deploy() {
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
            "428896412bda8530282a7b8fca5d20b2a73f30037612ca3a31750cf3bf0e976a",
        )
        .unwrap(),
        proxy: None,
    }];

    assert_eq!(contract_ids, expected)
}

#[tokio::test]
async fn deploy_fresh_proxy() {
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
            "428896412bda8530282a7b8fca5d20b2a73f30037612ca3a31750cf3bf0e976a",
        )
        .unwrap(),
        proxy: Some(
            ContractId::from_str(
                "fe084b07f5fd44f837d1fbf043671f0b27caef87503106b799b6a8b1ad5b30bd",
            )
            .unwrap(),
        ),
    };
    let expected = vec![impl_contract];

    assert_eq!(contract_ids, expected)
}

#[tokio::test]
async fn proxy_contract_re_routes_call() {
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
    let proxy_contract = contract_ids[0].proxy.unwrap();
    let impl_contract_id = contract_ids[0].id;
    // Make a contract call into proxy contract, and check if the initial
    // contract returns a true.
    let provider = Provider::connect(&node_url).await.unwrap();
    let secret_key = SecretKey::from_str(forc_client::constants::DEFAULT_PRIVATE_KEY).unwrap();
    let wallet_unlocked = WalletUnlocked::new_from_private_key(secret_key, Some(provider));

    abigen!(Contract(
        name = "ImplementationContract",
        abi = "forc-plugins/forc-client/test/data/standalone_contract/out/debug/standalone_contract-abi.json"
    ));

    let impl_contract_a = ImplementationContract::new(proxy_contract, wallet_unlocked.clone());
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
    assert_eq!(proxy_contract, proxy_contract_after_update);
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
