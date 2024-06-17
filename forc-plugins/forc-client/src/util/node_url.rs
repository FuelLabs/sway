use anyhow::{bail, Result};
use forc_pkg::manifest::Network;

use crate::NodeTarget;

use super::target::Target;

/// Returns the URL to use for connecting to Fuel Core node.
pub fn get_node_url(
    node_target: &NodeTarget,
    manifest_network: &Option<Network>,
) -> Result<String> {
    let node_url = match (
        node_target.testnet,
        node_target.target.clone(),
        node_target.node_url.clone(),
    ) {
        (true, None, None) => Target::testnet().target_url(),
        (false, Some(target), None) => target.target_url(),
        (false, None, Some(node_url)) => node_url,
        (false, None, None) => manifest_network
            .as_ref()
            .map(|nw| &nw.url[..])
            .unwrap_or(crate::constants::NODE_URL)
            .to_string(),
        _ => bail!("Only one of `--testnet`, `--target`, or `--node-url` should be specified"),
    };

    Ok(node_url)
}

#[test]
fn test_get_node_url_testnet() {
    let input = NodeTarget {
        target: None,
        node_url: None,
        testnet: true,
    };

    let actual = get_node_url(&input, &None).unwrap();
    assert_eq!("https://testnet.fuel.network", actual);
}

#[test]
fn test_get_node_url_target_devnet() {
    let input = NodeTarget {
        target: Some(Target::Devnet),
        node_url: None,
        testnet: false,
    };
    let actual = get_node_url(&input, &None).unwrap();
    assert_eq!("https://devnet.fuel.network", actual);
}

#[test]
fn test_get_node_url_target_testnet() {
    let input = NodeTarget {
        target: Some(Target::Testnet),
        node_url: None,
        testnet: false,
    };

    let actual = get_node_url(&input, &None).unwrap();
    assert_eq!("https://testnet.fuel.network", actual);
}

#[test]
fn test_get_node_url_beta5() {
    let input = NodeTarget {
        target: Some(Target::Beta5),
        node_url: None,
        testnet: false,
    };
    let actual = get_node_url(&input, &None).unwrap();
    assert_eq!("https://beta-5.fuel.network", actual);
}

#[test]
fn test_get_node_url_beta4() {
    let input = NodeTarget {
        target: None,
        node_url: Some("https://beta-4.fuel.network".to_string()),
        testnet: false,
    };
    let actual = get_node_url(&input, &None).unwrap();
    assert_eq!("https://beta-4.fuel.network", actual);
}

#[test]
fn test_get_node_url_url_beta4_manifest() {
    let network = Network {
        url: "https://beta-4.fuel.network".to_string(),
    };
    let input = NodeTarget {
        target: None,
        node_url: None,
        testnet: false,
    };

    let actual = get_node_url(&input, &Some(network)).unwrap();
    assert_eq!("https://beta-4.fuel.network", actual);
}

#[test]
fn test_get_node_url_default() {
    let input = NodeTarget {
        target: None,
        node_url: None,
        testnet: false,
    };

    let actual = get_node_url(&input, &None).unwrap();
    assert_eq!("http://127.0.0.1:4000", actual);
}

#[test]
fn test_get_node_url_beta3() {
    let input = NodeTarget {
        target: Some(Target::Beta3),
        node_url: None,
        testnet: false,
    };
    let actual = get_node_url(&input, &None).unwrap();
    assert_eq!("https://beta-3.fuel.network", actual);
}

#[test]
fn test_get_node_url_local() {
    let input = NodeTarget {
        target: Some(Target::Local),
        node_url: None,
        testnet: false,
    };
    let actual = get_node_url(&input, &None).unwrap();
    assert_eq!("http://127.0.0.1:4000", actual);
}

#[test]
#[should_panic(
    expected = "Only one of `--testnet`, `--target`, or `--node-url` should be specified"
)]
fn test_get_node_url_local_testnet() {
    let input = NodeTarget {
        target: Some(Target::Local),
        node_url: None,
        testnet: true,
    };
    get_node_url(&input, &None).unwrap();
}

#[test]
#[should_panic(
    expected = "Only one of `--testnet`, `--target`, or `--node-url` should be specified"
)]
fn test_get_node_url_same_url() {
    let input = NodeTarget {
        target: Some(Target::Beta3),
        node_url: Some("beta-3.fuel.network".to_string()),
        testnet: false,
    };
    get_node_url(&input, &None).unwrap();
}
