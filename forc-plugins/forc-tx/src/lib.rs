//! A simple tool for constructing transactions from the command line.

use anyhow::{bail, Context};
use clap::Parser;
use devault::Devault;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The top-level `forc tx` command.
#[derive(Debug, Parser, Deserialize, Serialize)]
#[clap(about, version)]
pub struct Command {
    #[clap(long, short = 'o')]
    pub output_path: Option<PathBuf>,
    #[clap(subcommand)]
    pub tx: Transaction,
}

/// Construct a transaction.
#[derive(Debug, Parser, Deserialize, Serialize)]
#[clap(name = "transaction")]
pub enum Transaction {
    Create(Create),
    Script(Script),
    Mint(Mint),
}

/// Construct a `Create` transaction for deploying a contract.
#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct Create {
    #[clap(flatten)]
    pub gas: Gas,
    /// Block until which tx cannot be included.
    #[clap(long)]
    pub maturity: u32,
    /// Added salt used to derive the contract ID.
    #[clap(long)]
    pub salt: Option<fuel_tx::Salt>,
    /// Path to the contract bytecode.
    #[clap(long)]
    pub bytecode: PathBuf,
    /// Witness index of contract bytecode to create.
    #[clap(long, default_value_t = 0)]
    pub bytecode_witness_index: u8,
    /// Path to a JSON file with a list of storage slots to initialize (key, value).
    #[clap(long)]
    pub storage_slots: PathBuf,
    /// An arbitrary length string of hex-encoded bytes (e.g. "1F2E3D4C5B6A")
    ///
    /// Can be specified multiple times.
    #[clap(long = "witness", multiple = true, max_values = 255)]
    pub witnesses: Vec<String>,
    // Inputs and outputs must follow all other arguments and are parsed separately.
    #[clap(skip)]
    pub inputs: Vec<Input>,
    // Inputs and outputs must follow all other arguments and are parsed separately.
    #[clap(skip)]
    pub outputs: Vec<Output>,
}

/// Construct a `Mint` transaction for emulating a block producer.
#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct Mint {
    /// The location of the `Mint` transaction in the block.
    #[clap(long)]
    pub tx_ptr: fuel_tx::TxPointer,
    // Outputs must follow all other arguments and are parsed separately.
    #[clap(skip)]
    pub outputs: Vec<Output>,
}

/// Construct a `Script` transaction for running a script.
#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct Script {
    #[clap(flatten)]
    pub gas: Gas,
    /// Block until which tx cannot be included.
    #[clap(long)]
    pub maturity: u32,
    /// Script to execute.
    #[clap(long)]
    pub bytecode: PathBuf,
    /// Script input data (parameters). Specified file is loaded as raw bytes.
    #[clap(long)]
    pub data: PathBuf,
    /// Merkle root of receipts.
    #[clap(long)]
    pub receipts_root: fuel_tx::Bytes32,
    /// An arbitrary length string of hex-encoded bytes (e.g. "1F2E3D4C5B6A")
    ///
    /// Can be specified multiple times.
    #[clap(long = "witness", multiple = true, max_values = 255)]
    pub witnesses: Vec<String>,
    // Inputs and outputs must follow all other arguments and are parsed separately.
    #[clap(skip)]
    pub inputs: Vec<Input>,
    // Inputs and outputs must follow all other arguments and are parsed separately.
    #[clap(skip)]
    pub outputs: Vec<Output>,
}

/// Flag set for specifying gas price and limit.
#[derive(Debug, Devault, Parser, Deserialize, Serialize)]
pub struct Gas {
    /// Gas price for the transaction.
    #[clap(long = "gas-price", default_value_t = 0)]
    #[devault("0")]
    pub price: u64,
    /// Gas limit for the transaction.
    #[clap(long = "gas-limit", default_value_t = fuel_tx::ConsensusParameters::DEFAULT.max_gas_per_tx)]
    #[devault("fuel_tx::ConsensusParameters::DEFAULT.max_gas_per_tx")]
    pub limit: u64,
}

/// Transaction input.
#[derive(Debug, Parser, Deserialize, Serialize)]
#[clap(name = "input")]
pub enum Input {
    Coin(InputCoin),
    Contract(InputContract),
    Message(InputMessage),
}

#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct InputCoin {
    /// Hash of the unspent transaction.
    #[clap(long)]
    pub utxo_id: fuel_tx::UtxoId,
    /// Index of transaction output.
    #[clap(long)]
    pub output_ix: u8,
    /// Owning address or predicate root.
    #[clap(long)]
    pub owner: fuel_tx::Address,
    /// Amount of coins.
    #[clap(long)]
    pub amount: u64,
    /// Asset ID of the coins.
    #[clap(long)]
    pub asset_id: fuel_tx::AssetId,
    /// Points to the TX whose output is being spent. Includes block height, tx index.
    #[clap(long)]
    pub tx_ptr: fuel_tx::TxPointer,
    /// Index of witness that authorizes spending the coin.
    #[clap(long)]
    pub witness_ix: Option<u8>,
    /// UTXO being spent must have been created at least this many blocks ago.
    #[clap(long)]
    pub maturity: u32,
    #[clap(flatten)]
    pub predicate: Predicate,
}

#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct InputContract {
    /// Hash of the unspent transaction.
    #[clap(long)]
    pub utxo_id: fuel_tx::UtxoId,
    /// Index of transaction output.
    #[clap(long)]
    pub output_ix: u8,
    /// Root of the amount of coins owned by the contract before transaction execution.
    #[clap(long)]
    pub balance_root: fuel_tx::Bytes32,
    /// State root of contract before transaction execution.
    #[clap(long)]
    pub state_root: fuel_tx::Bytes32,
    /// Points to the TX whose output is being spent. Includes block height, tx index.
    #[clap(long)]
    pub tx_ptr: fuel_tx::TxPointer,
    /// The ID of the contract.
    #[clap(long)]
    pub contract_id: fuel_tx::ContractId,
}

#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct InputMessage {
    /// The message ID as described here.
    #[clap(long)]
    pub msg_id: fuel_tx::MessageId,
    /// The address of the message sender.
    #[clap(long)]
    pub sender: fuel_tx::Address,
    /// The address or predicate root of the message recipient.
    #[clap(long)]
    pub recipient: fuel_tx::Address,
    /// Amount of base asset coins sent with message.
    #[clap(long)]
    pub amount: u64,
    /// The message nonce.
    #[clap(long)]
    pub nonce: u64,
    /// The message data.
    #[clap(long)]
    pub msg_data: PathBuf,
    /// Index of witness that authorizes the message.
    #[clap(long)]
    pub witness_ix: Option<u8>,
    #[clap(flatten)]
    pub predicate: Predicate,
}

/// Grouped arguments related to an input's predicate.
#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct Predicate {
    /// The predicate bytecode.
    #[clap(long = "predicate")]
    pub bytecode: Option<PathBuf>,
    /// The predicate's input data (parameters). Specified file is loaded as raw bytes.
    #[clap(long = "predicate-data")]
    pub data: Option<PathBuf>,
}

/// The location of the transaction in the block.
#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct TxPointer {
    /// The transaction block height.
    #[clap(long = "tx-ptr-block-height")]
    pub block_height: u32,
    /// Transaction index.
    #[clap(long = "tx-ptr-ix")]
    pub tx_ix: u16,
}

/// Transaction output.
#[derive(Debug, Parser, Deserialize, Serialize)]
#[clap(name = "output")]
pub enum Output {
    Coin(OutputCoin),
    Contract(OutputContract),
    Message(OutputMessage),
    Change(OutputChange),
    Variable(OutputVariable),
    ContractCreated(OutputContractCreated),
}

#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct OutputCoin {
    /// Hash of the unspent transaction.
    #[clap(long)]
    pub to: fuel_tx::Address,
    /// Amount of coins.
    #[clap(long)]
    pub amount: fuel_tx::Word,
    /// Asset ID of the coins.
    #[clap(long)]
    pub asset_id: fuel_tx::AssetId,
}

#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct OutputContract {
    /// Index of input contract.
    #[clap(long)]
    pub input_ix: u8,
    /// Root of amount of coins owned by contract after transaction execution.
    #[clap(long)]
    pub balance_root: fuel_tx::Bytes32,
    /// State root of contract after transaction execution.
    #[clap(long)]
    pub state_root: fuel_tx::Bytes32,
}

#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct OutputMessage {
    /// The address of the message recipient.
    #[clap(long)]
    pub recipient: fuel_tx::Address,
    /// The amount of asset coins sent with message.
    #[clap(long)]
    pub amount: fuel_tx::Word,
}

#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct OutputChange {
    /// Receiving address or predicate root.
    #[clap(long)]
    pub to: fuel_tx::Address,
    /// Amount of coins to send.
    #[clap(long)]
    pub amount: fuel_tx::Word,
    /// Asset ID of coins.
    #[clap(long)]
    pub asset_id: fuel_tx::AssetId,
}

#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct OutputVariable {
    /// Receiving address or predicate root.
    #[clap(long)]
    pub to: fuel_tx::Address,
    /// Amount of coins to send.
    #[clap(long)]
    pub amount: fuel_tx::Word,
    /// Asset ID of coins.
    #[clap(long)]
    pub asset_id: fuel_tx::AssetId,
}

#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct OutputContractCreated {
    /// Contract ID
    #[clap(long)]
    pub contract_id: fuel_tx::ContractId,
    /// Initial state root of contract.
    #[clap(long)]
    pub state_root: fuel_tx::Bytes32,
}

impl Command {
    /// Parse a full `Transaction` including trailing inputs and outputs.
    pub fn try_parse() -> anyhow::Result<Self> {
        Self::try_parse_from_args(std::env::args())
    }

    /// Parse a full `Transaction` including trailing inputs and outputs from an iterator yielding
    /// whitespace-separate string arguments.
    pub fn try_parse_from_args(args: impl IntoIterator<Item = String>) -> anyhow::Result<Self> {
        const INPUT: &str = "input";
        const OUTPUT: &str = "output";

        fn is_input_or_output(s: &str) -> bool {
            s == INPUT || s == OUTPUT
        }

        fn push_input(cmd: &mut Transaction, input: Input) -> anyhow::Result<()> {
            match cmd {
                Transaction::Create(ref mut create) => create.inputs.push(input),
                Transaction::Script(ref mut script) => script.inputs.push(input),
                Transaction::Mint(_) => {
                    bail!("Found argument 'input' which isn't valid for a Mint transaction");
                }
            }
            Ok(())
        }

        fn push_output(cmd: &mut Transaction, output: Output) {
            match cmd {
                Transaction::Create(ref mut create) => create.outputs.push(output),
                Transaction::Script(ref mut script) => script.outputs.push(output),
                Transaction::Mint(ref mut mint) => mint.outputs.push(output),
            }
        }

        let mut args = args.into_iter().peekable();

        // Collect args until the first `input` or `output` is reached.
        let mut cmd = {
            let cmd_args = std::iter::from_fn(|| args.next_if(|s| !is_input_or_output(s)));
            Command::try_parse_from(cmd_args)?
        };

        // The remaining args (if any) are the inputs and outputs.
        while let Some(arg) = args.next() {
            let args_til_next = std::iter::once(arg.clone()).chain(std::iter::from_fn(|| {
                args.next_if(|s| !is_input_or_output(s))
            }));
            match &arg[..] {
                INPUT => {
                    let input =
                        Input::try_parse_from(args_til_next).context("failed to parse input")?;
                    push_input(&mut cmd.tx, input)?
                }
                OUTPUT => {
                    let output =
                        Output::try_parse_from(args_til_next).context("failed to parse output")?;
                    push_output(&mut cmd.tx, output)
                }
                arg => bail!("unexpected argument {arg}, expected 'input' or 'output'"),
            }
        }

        // If there are args remaining, report them.
        if args.peek().is_some() {
            bail!("Unexpected remaining args: {:?}", args.collect::<Vec<_>>());
        }

        Ok(cmd)
    }
}

impl TryFrom<Transaction> for fuel_tx::Transaction {
    type Error = anyhow::Error;
    fn try_from(tx: Transaction) -> Result<Self, Self::Error> {
        let tx = match tx {
            Transaction::Create(create) => Self::Create(<_>::try_from(create)?),
            Transaction::Script(script) => Self::Script(<_>::try_from(script)?),
            Transaction::Mint(mint) => Self::Mint(mint.into()),
        };
        Ok(tx)
    }
}

impl TryFrom<Create> for fuel_tx::Create {
    type Error = anyhow::Error;
    fn try_from(create: Create) -> Result<Self, Self::Error> {
        let storage_slots = {
            let file = std::fs::File::open(create.storage_slots)?;
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).context("failed to parse storage_slots file")?
        };
        let inputs = create
            .inputs
            .into_iter()
            .map(fuel_tx::Input::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let outputs = create
            .outputs
            .into_iter()
            .map(fuel_tx::Output::from)
            .collect();
        let witnesses = create
            .witnesses
            .into_iter()
            .map(|s| fuel_tx::Witness::from(s.as_bytes()))
            .collect();
        let create = fuel_tx::Transaction::create(
            create.gas.price,
            create.gas.limit,
            // TODO: `fuel_tx` create shouldn't accept `Word`: spec says `u32`.
            create.maturity as fuel_tx::Word,
            create.bytecode_witness_index,
            create.salt.unwrap_or_default(),
            storage_slots,
            inputs,
            outputs,
            witnesses,
        );
        Ok(create)
    }
}

impl TryFrom<Script> for fuel_tx::Script {
    type Error = anyhow::Error;
    fn try_from(script: Script) -> Result<Self, Self::Error> {
        let script_bytecode = std::fs::read(script.bytecode)?;
        let script_data = std::fs::read(script.data)?;
        let inputs = script
            .inputs
            .into_iter()
            .map(fuel_tx::Input::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let outputs = script
            .outputs
            .into_iter()
            .map(fuel_tx::Output::from)
            .collect();
        let witnesses = script
            .witnesses
            .into_iter()
            .map(|s| fuel_tx::Witness::from(s.as_bytes()))
            .collect();
        let script = fuel_tx::Transaction::script(
            script.gas.price,
            script.gas.limit,
            // TODO: `fuel_tx` create shouldn't accept `Word`: spec says `u32`.
            script.maturity as fuel_tx::Word,
            script_bytecode,
            script_data,
            inputs,
            outputs,
            witnesses,
        );
        Ok(script)
    }
}

impl From<Mint> for fuel_tx::Mint {
    fn from(mint: Mint) -> Self {
        let outputs = mint
            .outputs
            .into_iter()
            .map(fuel_tx::Output::from)
            .collect();
        fuel_tx::Transaction::mint(mint.tx_ptr, outputs)
    }
}

impl TryFrom<Input> for fuel_tx::Input {
    type Error = anyhow::Error;
    fn try_from(input: Input) -> Result<Self, Self::Error> {
        let input = match input {
            Input::Coin(coin) => {
                let InputCoin {
                    utxo_id,
                    // TODO: Should this be verified / checked in some way?
                    output_ix: _,
                    owner,
                    amount,
                    asset_id,
                    tx_ptr: tx_pointer,
                    maturity,
                    predicate,
                    witness_ix,
                } = coin;
                match (witness_ix, predicate.bytecode, predicate.data) {
                    (Some(witness_index), None, None) => fuel_tx::Input::CoinSigned {
                        utxo_id,
                        owner,
                        amount,
                        asset_id,
                        tx_pointer,
                        maturity: maturity as fuel_tx::Word,
                        witness_index,
                    },
                    (None, Some(predicate), Some(predicate_data)) => {
                        fuel_tx::Input::CoinPredicate {
                            utxo_id,
                            owner,
                            amount,
                            asset_id,
                            tx_pointer,
                            maturity: maturity as fuel_tx::Word,
                            predicate: std::fs::read(predicate)?,
                            predicate_data: std::fs::read(predicate_data)?,
                        }
                    }
                    _ => bail!("input coin accepts either witness index or predicate, not both"),
                }
            }

            Input::Contract(contract) => fuel_tx::Input::Contract {
                utxo_id: contract.utxo_id,
                balance_root: contract.balance_root,
                state_root: contract.state_root,
                tx_pointer: contract.tx_ptr,
                contract_id: contract.contract_id,
            },

            Input::Message(msg) => {
                let InputMessage {
                    msg_id: message_id,
                    sender,
                    recipient,
                    amount,
                    nonce,
                    msg_data,
                    witness_ix,
                    predicate,
                } = msg;
                let data = std::fs::read(msg_data)?;
                match (witness_ix, predicate.bytecode, predicate.data) {
                    (Some(witness_index), None, None) => fuel_tx::Input::MessageSigned {
                        message_id,
                        sender,
                        recipient,
                        amount,
                        nonce,
                        data,
                        witness_index,
                    },
                    (None, Some(predicate), Some(predicate_data)) => {
                        fuel_tx::Input::MessagePredicate {
                            message_id,
                            sender,
                            recipient,
                            amount,
                            nonce,
                            data,
                            predicate: std::fs::read(predicate)?,
                            predicate_data: std::fs::read(predicate_data)?,
                        }
                    }
                    _ => bail!("input message accepts either witness index or predicate, not both"),
                }
            }
        };
        Ok(input)
    }
}

impl From<Output> for fuel_tx::Output {
    fn from(output: Output) -> Self {
        match output {
            Output::Coin(coin) => fuel_tx::Output::Coin {
                to: coin.to,
                amount: coin.amount,
                asset_id: coin.asset_id,
            },
            Output::Contract(contract) => fuel_tx::Output::Contract {
                input_index: contract.input_ix,
                balance_root: contract.balance_root,
                state_root: contract.state_root,
            },
            Output::Message(msg) => fuel_tx::Output::Message {
                recipient: msg.recipient,
                amount: msg.amount,
            },
            Output::Change(change) => fuel_tx::Output::Change {
                to: change.to,
                amount: change.amount,
                asset_id: change.asset_id,
            },
            Output::Variable(variable) => fuel_tx::Output::Variable {
                to: variable.to,
                amount: variable.amount,
                asset_id: variable.asset_id,
            },
            Output::ContractCreated(contract_created) => fuel_tx::Output::ContractCreated {
                contract_id: contract_created.contract_id,
                state_root: contract_created.state_root,
            },
        }
    }
}

#[test]
fn test_parse_create() {
    let cmd = r#"
        forc-tx create
            --bytecode ./my-contract/out/debug/my-contract.bin
            --storage-slots ./my-contract/out/debug/my-contract-storage_slots.json
            --gas-limit 100
            --gas-price 0
            --maturity 0
            --witness ADFD
            --witness DFDA
    "#;
    dbg!(Command::try_parse_from_args(cmd.split_whitespace().map(|s| s.to_string())).unwrap());
}

#[test]
fn test_parse_script() {
    let receipts_root = fuel_tx::Bytes32::default();
    let cmd = format!(
        r#"
        forc-tx script
            --bytecode ./my-script/out/debug/my-script.bin
            --data ./my-script.dat
            --gas-limit 100
            --gas-price 0
            --maturity 0
            --receipts-root {receipts_root}
            --witness ADFD
            --witness DFDA
    "#
    );
    dbg!(Command::try_parse_from_args(cmd.split_whitespace().map(|s| s.to_string())).unwrap());
}

#[test]
fn test_parse_mint_coin() {
    let tx_ptr = fuel_tx::TxPointer::default();
    let address = fuel_tx::Address::default();
    let asset_id = fuel_tx::AssetId::default();
    let cmd = format!(
        r#"
        forc-tx mint --tx-ptr {tx_ptr:X} output coin --to {address} --amount 100 --asset-id {asset_id}
    "#
    );
    dbg!(Command::try_parse_from_args(cmd.split_whitespace().map(|s| s.to_string())).unwrap());
}

#[test]
fn test_parse_create_inputs_outputs() {
    let address = fuel_tx::Address::default();
    let asset_id = fuel_tx::AssetId::default();
    let tx_ptr = fuel_tx::TxPointer::default();
    let balance_root = fuel_tx::Bytes32::default();
    let state_root = fuel_tx::Bytes32::default();
    let contract_id = fuel_tx::ContractId::default();
    let msg_id = fuel_tx::MessageId::default();
    let sender = fuel_tx::Address::default();
    let recipient = fuel_tx::Address::default();
    let args = format!(
        r#"
        forc-tx create
            --bytecode ./my-contract/out/debug/my-contract.bin
            --storage-slots ./my-contract/out/debug/my-contract-storage_slots.json
            --gas-limit 100
            --gas-price 0
            --maturity 0
            --witness ADFD
            --witness DFDA
            input coin
                --utxo-id 0
                --output-ix 0
                --owner {address}
                --amount 100
                --asset-id {asset_id}
                --tx-ptr {tx_ptr:X}
                --witness-ix 0
                --maturity 0
                --predicate ./my-predicate/out/debug/my-predicate.bin
                --predicate-data ./my-predicate.dat
            input contract
                --utxo-id 1
                --output-ix 1
                --balance-root {balance_root}
                --state-root {state_root}
                --tx-ptr {tx_ptr:X}
                --contract-id {contract_id}
            input message
                --msg-id {msg_id}
                --sender {sender}
                --recipient {recipient}
                --amount 1
                --nonce 1234
                --witness-ix 1
                --msg-data ./message.dat
                --predicate ./my-predicate2/out/debug/my-predicate2.bin
                --predicate-data ./my-predicate2.dat
            output coin
                --to {address}
                --amount 100
                --asset-id {asset_id}
            output contract
                --input-ix 1
                --balance-root {balance_root}
                --state-root {state_root}
            output message
                --recipient {address}
                --amount 100
            output change
                --to {address}
                --amount 100
                --asset-id {asset_id}
            output variable
                --to {address}
                --amount 100
                --asset-id {asset_id}
            output contract-created
                --contract-id {contract_id}
                --state-root {state_root}
    "#
    );
    dbg!(Command::try_parse_from_args(args.split_whitespace().map(|s| s.to_string())).unwrap());
}
