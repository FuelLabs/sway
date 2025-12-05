//! A simple tool for constructing transactions from the command line.

use clap::{Args, Parser};
use devault::Devault;
use forc_util::tx_utils::Salt;
use fuel_tx::{
    output,
    policies::{Policies, PolicyType},
    Buildable, Chargeable, ConsensusParameters,
};
use fuels_core::types::transaction::TxPolicies;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

forc_types::cli_examples! {
    {
        // This parser has a custom parser
        super::Command::try_parse_from_args
    } {
    [ Script example => r#"forc tx script --bytecode "{path}/out/debug/name.bin" --data "{path}/data.bin" \
        --receipts-root 0x2222222222222222222222222222222222222222222222222222222222222222"# ]
    [ Multiple inputs => r#"forc tx create --bytecode "{name}/out/debug/name.bin"
        --storage-slots "{path}/out/debug/name-storage_slots.json"
        --script-gas-limit 100 \
        --gas-price 0 \
        --maturity 0 \
        --witness ADFD \
        --witness DFDA \
        input coin \
            --utxo-id 0 \
            --output-ix 0 \
            --owner 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --amount 100 \
            --asset-id 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --tx-ptr 89ACBDEFBDEF \
            --witness-ix 0 \
            --maturity 0 \
        input contract \
            --utxo-id 1 \
            --output-ix 1 \
            --balance-root 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --state-root 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --tx-ptr 89ACBDEFBDEF \
            --contract-id 0xCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC \
        output coin \
            --to 0x2222222222222222222222222222222222222222222222222222222222222222 \
            --amount 100 \
            --asset-id 0x0000000000000000000000000000000000000000000000000000000000000000 \
        output contract \
            --input-ix 1 \
            --balance-root 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --state-root 0x0000000000000000000000000000000000000000000000000000000000000000 \
        output change \
            --to 0x2222222222222222222222222222222222222222222222222222222222222222 \
            --amount 100 \
            --asset-id 0x0000000000000000000000000000000000000000000000000000000000000000 \
        output variable \
            --to 0x2222222222222222222222222222222222222222222222222222222222222222 \
            --amount 100 \
            --asset-id 0x0000000000000000000000000000000000000000000000000000000000000000 \
        output contract-created \
            --contract-id 0xCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC \
            --state-root 0x0000000000000000000000000000000000000000000000000000000000000000
        "#
    ]
    [ An example constructing a create transaction => r#"forc tx create \
        --bytecode {path}/out/debug/name.bin \
        --storage-slots {path}/out/debug/name-storage_slots.json \
        --script-gas-limit 100 \
        --gas-price 0 \
        --maturity 0 \
        --witness ADFD \
        --witness DFDA \
        input coin \
            --utxo-id 0 \
            --output-ix 0 \
            --owner 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --amount 100 \
            --asset-id 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --tx-ptr 89ACBDEFBDEF \
            --witness-ix 0 \
            --maturity 0 \
        input contract \
            --utxo-id 1 \
            --output-ix 1 \
            --balance-root 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --state-root 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --tx-ptr 89ACBDEFBDEF \
            --contract-id 0xCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC \
        input message \
            --sender 0x1111111111111111111111111111111111111111111111111111111111111111 \
            --recipient 0x2222222222222222222222222222222222222222222222222222222222222222 \
            --amount 1 \
            --nonce 0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB \
            --msg-data {path}/message.dat \
            --predicate {path}/my-predicate2.bin \
            --predicate-data {path}/my-predicate2.dat \
        output coin \
            --to 0x2222222222222222222222222222222222222222222222222222222222222222 \
            --amount 100 \
            --asset-id 0x0000000000000000000000000000000000000000000000000000000000000000 \
        output contract \
            --input-ix 1 \
            --balance-root 0x0000000000000000000000000000000000000000000000000000000000000000 \
            --state-root 0x0000000000000000000000000000000000000000000000000000000000000000 \
        output change \
            --to 0x2222222222222222222222222222222222222222222222222222222222222222 \
            --amount 100 \
            --asset-id 0x0000000000000000000000000000000000000000000000000000000000000000 \
        output variable \
            --to 0x2222222222222222222222222222222222222222222222222222222222222222 \
            --amount 100 \
            --asset-id 0x0000000000000000000000000000000000000000000000000000000000000000 \
        output contract-created \
            --contract-id 0xCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC \
            --state-root 0x0000000000000000000000000000000000000000000000000000000000000000"#
    ]
    }
}

/// The top-level `forc tx` command.
#[derive(Debug, Parser, Deserialize, Serialize)]
#[clap(about, version, after_help = help())]
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
}

/// Construct a `Create` transaction for deploying a contract.
#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct Create {
    #[clap(flatten)]
    pub gas: Gas,
    #[clap(flatten)]
    pub maturity: Maturity,
    #[clap(flatten)]
    pub salt: Salt,
    /// Path to the contract bytecode.
    #[clap(long)]
    pub bytecode: PathBuf,
    /// Witness index of contract bytecode to create.
    #[clap(long, default_value_t = 0)]
    pub bytecode_witness_index: u16,
    /// Path to a JSON file with a list of storage slots to initialize (key, value).
    #[clap(long)]
    pub storage_slots: PathBuf,
    /// An arbitrary length string of hex-encoded bytes (e.g. "1F2E3D4C5B6A")
    ///
    /// Can be specified multiple times.
    #[clap(long = "witness", num_args(0..255))]
    pub witnesses: Vec<String>,
    // Inputs and outputs must follow all other arguments and are parsed separately.
    #[clap(skip)]
    pub inputs: Vec<Input>,
    // Inputs and outputs must follow all other arguments and are parsed separately.
    #[clap(skip)]
    pub outputs: Vec<Output>,
}

/// Construct a `Script` transaction for running a script.
#[derive(Debug, Parser, Deserialize, Serialize)]
pub struct Script {
    #[clap(flatten)]
    pub gas: Gas,
    #[clap(flatten)]
    pub maturity: Maturity,
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
    #[clap(long = "witness", num_args(0..=255))]
    pub witnesses: Vec<String>,
    // Inputs and outputs must follow all other arguments and are parsed separately.
    #[clap(skip)]
    pub inputs: Vec<Input>,
    // Inputs and outputs must follow all other arguments and are parsed separately.
    #[clap(skip)]
    pub outputs: Vec<Output>,
}

/// Flag set for specifying gas price and limit.
#[derive(Debug, Devault, Clone, Parser, Deserialize, Serialize)]
pub struct Gas {
    /// Gas price for the transaction.
    #[clap(long = "gas-price")]
    pub price: Option<u64>,
    /// Gas limit for the transaction.
    #[clap(long = "script-gas-limit")]
    pub script_gas_limit: Option<u64>,
    /// Max fee for the transaction.
    #[clap(long)]
    pub max_fee: Option<u64>,
    /// The tip for the transaction.
    #[clap(long)]
    pub tip: Option<u64>,
}

/// Block until which tx cannot be included.
#[derive(Debug, Args, Default, Deserialize, Serialize)]
pub struct Maturity {
    /// Block height until which tx cannot be included.
    #[clap(long = "maturity", default_value_t = 0)]
    pub maturity: u32,
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
    pub witness_ix: Option<u16>,
    /// UTXO being spent must have been created at least this many blocks ago.
    #[clap(long)]
    pub maturity: u32,
    /// Gas used by predicates.
    #[clap(long, default_value_t = 0)]
    pub predicate_gas_used: u64,
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
    pub nonce: fuel_types::Nonce,
    /// The message data.
    #[clap(long)]
    pub msg_data: PathBuf,
    /// Index of witness that authorizes the message.
    #[clap(long)]
    pub witness_ix: Option<u16>,
    /// Gas used by predicates.
    #[clap(long, default_value_t = 0)]
    pub predicate_gas_used: u64,
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
    pub input_ix: u16,
    /// Root of amount of coins owned by contract after transaction execution.
    #[clap(long)]
    pub balance_root: fuel_tx::Bytes32,
    /// State root of contract after transaction execution.
    #[clap(long)]
    pub state_root: fuel_tx::Bytes32,
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

/// Errors that can occur while parsing the `Command`.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to parse the command")]
    Command {
        #[source]
        err: clap::Error,
    },
    #[error("Failed to parse transaction `input`")]
    Input {
        #[source]
        err: clap::Error,
    },
    #[error("Failed to parse transaction `output`")]
    Output {
        #[source]
        err: clap::Error,
    },
    #[error("Unrecognized argument {arg:?}, expected `input` or `output`")]
    UnrecognizedArgumentExpectedInputOutput { arg: String, remaining: Vec<String> },
    #[error("Found argument `input` which isn't valid for a mint transaction")]
    MintTxHasInput,
}

/// Errors that can occur during conversion from the CLI transaction
/// representation to the `fuel-tx` representation.
#[derive(Debug, Error)]
pub enum ConvertTxError {
    #[error("failed to convert create transaction")]
    Create(#[from] ConvertCreateTxError),
    #[error("failed to convert script transaction")]
    Script(#[from] ConvertScriptTxError),
}

/// Errors that can occur during "create" transaction conversion.
#[derive(Debug, Error)]
pub enum ConvertCreateTxError {
    #[error("failed to open `--storage-slots` from {path:?}")]
    StorageSlotsOpen {
        path: PathBuf,
        #[source]
        err: std::io::Error,
    },
    #[error("failed to deserialize storage slots file")]
    StorageSlotsDeserialize(#[source] serde_json::Error),
    #[error("failed to convert an input")]
    Input(#[from] ConvertInputError),
}

/// Errors that can occur during "script" transaction conversion.
#[derive(Debug, Error)]
pub enum ConvertScriptTxError {
    #[error("failed to read `--bytecode` from {path:?}")]
    BytecodeRead {
        path: PathBuf,
        #[source]
        err: std::io::Error,
    },
    #[error("failed to read `--data` from {path:?}")]
    DataRead {
        path: PathBuf,
        #[source]
        err: std::io::Error,
    },
    #[error("failed to convert an input")]
    Input(#[from] ConvertInputError),
}

/// Errors that can occur during transaction input conversion.
#[derive(Debug, Error)]
pub enum ConvertInputError {
    #[error("failed to read `--msg-data` from {path:?}")]
    MessageDataRead {
        path: PathBuf,
        #[source]
        err: std::io::Error,
    },
    #[error("failed to read `--predicate` from {path:?}")]
    PredicateRead {
        path: PathBuf,
        #[source]
        err: std::io::Error,
    },
    #[error("failed to read `--predicate-data` from {path:?}")]
    PredicateDataRead {
        path: PathBuf,
        #[source]
        err: std::io::Error,
    },
    #[error("input accepts either witness index or predicate, not both")]
    WitnessPredicateMismatch,
}

impl ParseError {
    /// Print the error with clap's fancy formatting.
    pub fn print(&self) -> Result<(), clap::Error> {
        match self {
            ParseError::Command { err } => {
                err.print()?;
            }
            ParseError::Input { err } => {
                err.print()?;
            }
            ParseError::Output { err } => {
                err.print()?;
            }
            ParseError::UnrecognizedArgumentExpectedInputOutput { .. } => {
                use clap::CommandFactory;
                // Create a type as a hack to produce consistent-looking clap help output.
                #[derive(Parser)]
                enum ForcTxIo {
                    #[clap(subcommand)]
                    Input(Input),
                    #[clap(subcommand)]
                    Output(Output),
                }
                println!("{self}\n");
                ForcTxIo::command().print_long_help()?;
            }
            ParseError::MintTxHasInput => {
                println!("{self}");
            }
        }
        Ok(())
    }
}

impl Command {
    /// Emulates `clap::Parser::parse` behaviour, but returns the parsed inputs and outputs.
    ///
    /// If parsing fails, prints the error along with the help output and exits with an error code.
    ///
    /// We provide this custom `parse` function solely due to clap's limitations around parsing
    /// trailing subcommands.
    pub fn parse() -> Self {
        let err = match Self::try_parse() {
            Err(err) => err,
            Ok(cmd) => return cmd,
        };
        let _ = err.print();
        std::process::exit(1);
    }

    /// Parse a full `Transaction` including trailing inputs and outputs.
    pub fn try_parse() -> Result<Self, ParseError> {
        Self::try_parse_from_args(std::env::args())
    }

    /// Parse a full `Transaction` including trailing inputs and outputs from an iterator yielding
    /// whitespace-separate string arguments.
    pub fn try_parse_from_args(args: impl IntoIterator<Item = String>) -> Result<Self, ParseError> {
        const INPUT: &str = "input";
        const OUTPUT: &str = "output";

        fn is_input_or_output(s: &str) -> bool {
            s == INPUT || s == OUTPUT
        }

        fn push_input(cmd: &mut Transaction, input: Input) -> Result<(), ParseError> {
            match cmd {
                Transaction::Create(ref mut create) => create.inputs.push(input),
                Transaction::Script(ref mut script) => script.inputs.push(input),
            }
            Ok(())
        }

        fn push_output(cmd: &mut Transaction, output: Output) {
            match cmd {
                Transaction::Create(ref mut create) => create.outputs.push(output),
                Transaction::Script(ref mut script) => script.outputs.push(output),
            }
        }

        let mut args = args.into_iter().peekable();

        // Collect args until the first `input` or `output` is reached.
        let mut cmd = {
            let cmd_args = std::iter::from_fn(|| args.next_if(|s| !is_input_or_output(s)));
            Command::try_parse_from(cmd_args).map_err(|err| ParseError::Command { err })?
        };

        // The remaining args (if any) are the inputs and outputs.
        while let Some(arg) = args.next() {
            let args_til_next = std::iter::once(arg.clone()).chain(std::iter::from_fn(|| {
                args.next_if(|s| !is_input_or_output(s))
            }));
            match &arg[..] {
                INPUT => {
                    let input = Input::try_parse_from(args_til_next)
                        .map_err(|err| ParseError::Input { err })?;
                    push_input(&mut cmd.tx, input)?
                }
                OUTPUT => {
                    let output = Output::try_parse_from(args_til_next)
                        .map_err(|err| ParseError::Output { err })?;
                    push_output(&mut cmd.tx, output)
                }
                arg => {
                    return Err(ParseError::UnrecognizedArgumentExpectedInputOutput {
                        arg: arg.to_string(),
                        remaining: args.collect(),
                    })
                }
            }
        }

        // If there are args remaining, report them.
        if args.peek().is_some() {
            return Err(ParseError::UnrecognizedArgumentExpectedInputOutput {
                arg: args.peek().unwrap().to_string(),
                remaining: args.collect(),
            });
        }

        Ok(cmd)
    }
}

impl TryFrom<Transaction> for fuel_tx::Transaction {
    type Error = ConvertTxError;
    fn try_from(tx: Transaction) -> Result<Self, Self::Error> {
        let tx = match tx {
            Transaction::Create(create) => Self::Create(<_>::try_from(create)?),
            Transaction::Script(script) => Self::Script(<_>::try_from(script)?),
        };
        Ok(tx)
    }
}

impl TryFrom<Create> for fuel_tx::Create {
    type Error = ConvertCreateTxError;
    fn try_from(create: Create) -> Result<Self, Self::Error> {
        let storage_slots = {
            let file = std::fs::File::open(&create.storage_slots).map_err(|err| {
                ConvertCreateTxError::StorageSlotsOpen {
                    path: create.storage_slots,
                    err,
                }
            })?;
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader)
                .map_err(ConvertCreateTxError::StorageSlotsDeserialize)?
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

        let maturity = (create.maturity.maturity != 0).then_some(create.maturity.maturity.into());
        let mut policies = Policies::default();
        policies.set(PolicyType::Tip, create.gas.price);
        policies.set(PolicyType::Maturity, maturity);

        let create = fuel_tx::Transaction::create(
            create.bytecode_witness_index,
            policies,
            create.salt.salt.unwrap_or_default(),
            storage_slots,
            inputs,
            outputs,
            witnesses,
        );

        Ok(create)
    }
}

impl TryFrom<Script> for fuel_tx::Script {
    type Error = ConvertScriptTxError;
    fn try_from(script: Script) -> Result<Self, Self::Error> {
        let script_bytecode =
            std::fs::read(&script.bytecode).map_err(|err| ConvertScriptTxError::BytecodeRead {
                path: script.bytecode,
                err,
            })?;

        let script_data =
            std::fs::read(&script.data).map_err(|err| ConvertScriptTxError::DataRead {
                path: script.data,
                err,
            })?;
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

        let mut policies = Policies::default().with_maturity(script.maturity.maturity.into());
        policies.set(PolicyType::Tip, script.gas.price);
        let mut script_tx = fuel_tx::Transaction::script(
            0, // Temporary value. Will be replaced below
            script_bytecode,
            script_data,
            policies,
            inputs,
            outputs,
            witnesses,
        );

        if let Some(script_gas_limit) = script.gas.script_gas_limit {
            script_tx.set_script_gas_limit(script_gas_limit)
        } else {
            let consensus_params = ConsensusParameters::default();
            // Get `max_gas` used by everything except the script execution. Add `1` because of rounding.
            let max_gas =
                script_tx.max_gas(consensus_params.gas_costs(), consensus_params.fee_params()) + 1;
            // Increase `script_gas_limit` to the maximum allowed value.
            script_tx.set_script_gas_limit(consensus_params.tx_params().max_gas_per_tx() - max_gas);
        }

        Ok(script_tx)
    }
}

impl TryFrom<Input> for fuel_tx::Input {
    type Error = ConvertInputError;
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
                    maturity: _,
                    predicate_gas_used,
                    predicate,
                    witness_ix,
                } = coin;
                match (witness_ix, predicate.bytecode, predicate.data) {
                    (Some(witness_index), None, None) => fuel_tx::Input::coin_signed(
                        utxo_id,
                        owner,
                        amount,
                        asset_id,
                        tx_pointer,
                        witness_index,
                    ),
                    (None, Some(predicate), Some(predicate_data)) => {
                        fuel_tx::Input::coin_predicate(
                            utxo_id,
                            owner,
                            amount,
                            asset_id,
                            tx_pointer,
                            predicate_gas_used,
                            std::fs::read(&predicate).map_err(|err| {
                                ConvertInputError::PredicateRead {
                                    path: predicate,
                                    err,
                                }
                            })?,
                            std::fs::read(&predicate_data).map_err(|err| {
                                ConvertInputError::PredicateDataRead {
                                    path: predicate_data,
                                    err,
                                }
                            })?,
                        )
                    }
                    _ => return Err(ConvertInputError::WitnessPredicateMismatch),
                }
            }

            Input::Contract(contract) => fuel_tx::Input::contract(
                contract.utxo_id,
                contract.balance_root,
                contract.state_root,
                contract.tx_ptr,
                contract.contract_id,
            ),

            Input::Message(msg) => {
                let InputMessage {
                    sender,
                    recipient,
                    amount,
                    nonce,
                    msg_data,
                    witness_ix,
                    predicate_gas_used,
                    predicate,
                } = msg;
                let data =
                    std::fs::read(&msg_data).map_err(|err| ConvertInputError::MessageDataRead {
                        path: msg_data,
                        err,
                    })?;
                match (witness_ix, predicate.bytecode, predicate.data) {
                    (Some(witness_index), None, None) => {
                        if data.is_empty() {
                            fuel_tx::Input::message_coin_signed(
                                sender,
                                recipient,
                                amount,
                                nonce,
                                witness_index,
                            )
                        } else {
                            fuel_tx::Input::message_data_signed(
                                sender,
                                recipient,
                                amount,
                                nonce,
                                witness_index,
                                data,
                            )
                        }
                    }
                    (None, Some(predicate), Some(predicate_data)) => {
                        let predicate = std::fs::read(&predicate).map_err(|err| {
                            ConvertInputError::PredicateRead {
                                path: predicate,
                                err,
                            }
                        })?;
                        let predicate_data = std::fs::read(&predicate_data).map_err(|err| {
                            ConvertInputError::PredicateDataRead {
                                path: predicate_data,
                                err,
                            }
                        })?;

                        if data.is_empty() {
                            fuel_tx::Input::message_coin_predicate(
                                sender,
                                recipient,
                                amount,
                                nonce,
                                predicate_gas_used,
                                predicate,
                                predicate_data,
                            )
                        } else {
                            fuel_tx::Input::message_data_predicate(
                                sender,
                                recipient,
                                amount,
                                nonce,
                                predicate_gas_used,
                                data,
                                predicate,
                                predicate_data,
                            )
                        }
                    }
                    _ => return Err(ConvertInputError::WitnessPredicateMismatch),
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
            Output::Contract(contract) => fuel_tx::Output::Contract(output::contract::Contract {
                input_index: contract.input_ix,
                balance_root: contract.balance_root,
                state_root: contract.state_root,
            }),
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

impl From<&Gas> for TxPolicies {
    fn from(gas: &Gas) -> Self {
        let mut policies = TxPolicies::default();
        if let Some(max_fee) = gas.max_fee {
            policies = policies.with_max_fee(max_fee);
        }
        if let Some(script_gas_limit) = gas.script_gas_limit {
            policies = policies.with_script_gas_limit(script_gas_limit);
        }
        if let Some(tip) = gas.tip {
            policies = policies.with_tip(tip);
        }
        policies
    }
}

#[test]
fn test_parse_create() {
    let cmd = r#"
        forc-tx create
            --bytecode ./my-contract/out/debug/my-contract.bin
            --storage-slots ./my-contract/out/debug/my-contract-storage_slots.json
            --script-gas-limit 100
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
            --script-gas-limit 100
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
fn test_parse_create_inputs_outputs() {
    let address = fuel_tx::Address::default();
    let asset_id = fuel_tx::AssetId::default();
    let tx_ptr = fuel_tx::TxPointer::default();
    let balance_root = fuel_tx::Bytes32::default();
    let state_root = fuel_tx::Bytes32::default();
    let contract_id = fuel_tx::ContractId::default();
    let nonce = fuel_types::Nonce::default();
    let sender = fuel_tx::Address::default();
    let recipient = fuel_tx::Address::default();
    let args = format!(
        r#"
        forc-tx create
            --bytecode ./my-contract/out/debug/my-contract.bin
            --storage-slots ./my-contract/out/debug/my-contract-storage_slots.json
            --script-gas-limit 100
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
                --sender {sender}
                --recipient {recipient}
                --amount 1
                --nonce {nonce}
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
