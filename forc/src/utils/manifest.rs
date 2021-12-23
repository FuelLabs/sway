use crate::utils::dependency::Dependency;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;

use common::constants::DEFAULT_NODE_URL;

// using https://github.com/rust-lang/cargo/blob/master/src/cargo/util/toml/mod.rs as the source of
// implementation strategy

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub project: Project,
    pub network: Option<Network>,
    pub tx_input: Option<Vec<TxInput>>,
    pub dependencies: Option<BTreeMap<String, Dependency>>,
}

impl Manifest {
    /// Given some inputs, constructs the most basic output set that satisfies validation.
    pub fn get_tx_inputs_and_outputs(
        &self,
    ) -> Result<(Vec<fuel_tx::Input>, Vec<fuel_tx::Output>), String> {
        let inputs = self
            .tx_input
            .clone()
            .unwrap_or_default()
            .iter()
            .map(TxInput::to_input)
            .collect::<Result<Vec<_>, _>>()?;
        let outputs = inputs
            .iter()
            .enumerate()
            .filter_map(construct_output_from_input)
            .collect::<Vec<_>>();
        Ok((inputs, outputs))
    }
}

fn construct_output_from_input((idx, input): (usize, &fuel_tx::Input)) -> Option<fuel_tx::Output> {
    match input {
        fuel_tx::Input::Contract {
            balance_root,
            state_root,
            ..
        } => Some(fuel_tx::Output::Contract {
            input_index: idx as u8, // probably safe unless a user inputs > u8::max inputs
            balance_root: *balance_root,
            state_root: *state_root,
        }),
        _ => None,
    }
}
/// This struct exists and is converted into a [fuel_tx::Input] because of limitations
/// of our toml library. It doesn't support directly deserializing [fuel_tx::Input].
///
/// It handles everything as optional strings and parses them in order to provide better error
/// messages.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TxInput {
    r#type: String,
    utxo_id: Option<String>,
    balance_root: Option<String>,
    state_root: Option<String>,
    contract_id: Option<String>,
    owner: Option<String>,
    amount: Option<String>,
    color: Option<String>,
    witness_index: Option<String>,
    maturity: Option<String>,
    predicate: Option<String>,
    predicate_data: Option<String>,
}

fn try_parse_bytes32(raw: &Option<String>, name: &str) -> Result<fuel_tx::Bytes32, String> {
    let mut raw = if let Some(raw) = raw {
        raw.to_string()
    } else {
        return Err(format!("Missing value for field {}.\nhelp: a tx-input entry in your Forc.toml manifest is missing a field named {}.", name, name));
    };
    if raw.len() > 2 && &raw[0..2] == "0x" {
        raw = (&raw[2..]).to_string();
    }
    Ok(TryFrom::try_from(
        hex::decode(&raw[..])
            .map_err(|_| format!(r#"Given value for "{}" ({}) is not valid."#, name, raw))?
            .as_slice(),
    )
    .unwrap())
}
fn try_parse_contract_id(raw: &Option<String>) -> Result<fuel_tx::ContractId, String> {
    let mut raw = if let Some(raw) = raw {
        raw.to_string()
    } else {
        return Err("Missing contract-id in manifest.".into());
    };
    if raw.len() > 2 && &raw[0..2] == "0x" {
        raw = (&raw[2..]).to_string();
    }
    Ok(TryFrom::try_from(
        hex::decode(&raw[..])
            .map_err(|_| {
                format!(
                    r#"In the manifest file (Forc.toml), the given value for "contract-id" in tx-inputs ({}) is not hexadecimal."#,
                    raw
                )
            })?
            .as_slice(),
    )
    .unwrap())
}
impl TxInput {
    pub fn to_input(&self) -> Result<fuel_tx::Input, String> {
        match self.r#type.to_lowercase().as_ref() {
            "contract" => Ok(fuel_tx::Input::Contract {
                utxo_id: try_parse_bytes32(&self.utxo_id, "utxo-id")?,
                balance_root: try_parse_bytes32(&self.balance_root, "balance-root")?,
                state_root: try_parse_bytes32(&self.state_root, "state-root")?,
                contract_id: try_parse_contract_id(&self.contract_id)?,
            }),
            "coin" => Err("Coin transaction inputs are not currently supported.".into()),
            a => Err(format!(
                r#"Expected tx input type of either "Contract" or "Coin", but received "{}""#,
                a
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    pub author: String,
    pub name: String,
    pub license: String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_entry() -> String {
    "main.sw".into()
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    #[serde(default = "default_url")]
    pub url: String,
}

fn default_url() -> String {
    DEFAULT_NODE_URL.into()
}

#[test]
fn try_parse() {
    println!(
        "{:#?}",
        toml::from_str::<Manifest>(&super::defaults::default_manifest("test_proj".into())).unwrap()
    )
}

#[test]
fn test_print_tx_inputs() {
    let mut default_manifest: Manifest =
        toml::from_str::<Manifest>(&super::defaults::default_manifest("test_proj".into())).unwrap();

    let input1 = TxInput {
        contract_id: Some(
            "0xeeb578f9e1ebfb5b78f8ff74352370c120bc8cacead1f5e4f9c74aafe0ca6bfd".into(),
        ),
        utxo_id: Some("blah".into()),
        ..Default::default()
    };
    let input2 = TxInput {
        contract_id: Some(
            "0xe7777777777bfb5b78f8ff74352370c120bc8cacead1f5e4f9c74aafe0ca6bfd".into(),
        ),
        utxo_id: Some("blah".into()),
        ..Default::default()
    };

    default_manifest.tx_input = Some(vec![input1, input2]);
    println!("{}", toml::to_string(&default_manifest).unwrap());
}
