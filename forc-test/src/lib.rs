use std::{collections::HashSet, fs, path::PathBuf, sync::Arc};

use forc_pkg as pkg;
use fuel_abi_types::error_codes::ErrorSignal;
use fuel_tx as tx;
use fuel_vm::checked_transaction::builder::TransactionBuilderExt;
use fuel_vm::gas::GasCosts;
use fuel_vm::{self as vm, fuel_asm, prelude::Instruction};
use pkg::TestPassCondition;
use pkg::{Built, BuiltPackage};
use rand::{Rng, SeedableRng};
use sway_core::BuildTarget;
use sway_types::Span;

/// The result of a `forc test` invocation.
#[derive(Debug)]
pub enum Tested {
    Package(Box<TestedPackage>),
    Workspace(Vec<TestedPackage>),
}

/// The result of testing a specific package.
#[derive(Debug)]
pub struct TestedPackage {
    pub built: Box<pkg::BuiltPackage>,
    /// The resulting `ProgramState` after executing the test.
    pub tests: Vec<TestResult>,
}

#[derive(Debug)]
pub struct TestDetails {
    /// The file that contains the test function.
    pub file_path: Arc<PathBuf>,
    /// The line number for the test declaration.
    pub line_number: usize,
}

/// The result of executing a single test within a single package.
#[derive(Debug)]
pub struct TestResult {
    /// The name of the function.
    pub name: String,
    /// The time taken for the test to execute.
    pub duration: std::time::Duration,
    /// The span for the function declaring this test.
    pub span: Span,
    /// The resulting state after executing the test function.
    pub state: vm::state::ProgramState,
    /// The required state of the VM for this test to pass.
    pub condition: pkg::TestPassCondition,
    /// Emitted `Recipt`s during the execution of the test.
    pub logs: Vec<fuel_tx::Receipt>,
    /// Gas used while executing this test.
    pub gas_used: u64,
}

const TEST_METADATA_SEED: u64 = 0x7E57u64;

/// A package or a workspace that has been built, ready for test execution.
pub enum BuiltTests {
    Package(PackageTests),
    Workspace(Vec<PackageTests>),
}

/// A built package ready for test execution.
///
/// If the built package is a contract, a second built package for the same contract without the
/// tests are also populated.
#[derive(Debug)]
pub enum PackageTests {
    Contract(ContractToTest),
    NonContract(pkg::BuiltPackage),
}

/// A built contract ready for test execution.
#[derive(Debug)]
pub struct ContractToTest {
    /// Tests included contract.
    pub pkg: pkg::BuiltPackage,
    /// Bytecode of the contract without tests.
    pub without_tests_bytecode: pkg::BuiltPackageBytecode,
}

/// The set of options provided to the `test` function.
#[derive(Default, Clone)]
pub struct Opts {
    pub pkg: pkg::PkgOpts,
    pub print: pkg::PrintOpts,
    pub minify: pkg::MinifyOpts,
    /// If set, outputs a binary file representing the script bytes.
    pub binary_outfile: Option<String>,
    /// If set, outputs source file mapping in JSON format
    pub debug_outfile: Option<String>,
    /// Build target to use.
    pub build_target: BuildTarget,
    /// Name of the build profile to use.
    /// If it is not specified, forc will use debug build profile.
    pub build_profile: Option<String>,
    /// Use release build plan. If a custom release plan is not specified, it is implicitly added to the manifest file.
    ///
    /// If --build-profile is also provided, forc omits this flag and uses provided build-profile.
    pub release: bool,
    /// Should warnings be treated as errors?
    pub error_on_warnings: bool,
    /// Output the time elapsed over each part of the compilation process.
    pub time_phases: bool,
}

/// The set of options provided for controlling logs printed for each test.
#[derive(Default, Clone)]
pub struct TestPrintOpts {
    pub pretty_print: bool,
    pub print_logs: bool,
}

/// The storage and the contract id (if a contract is being tested) for a test.
#[derive(Debug)]
struct TestSetup {
    storage: vm::storage::MemoryStorage,
    contract_id: Option<tx::ContractId>,
}

impl BuiltTests {
    /// Constructs a `PackageTests` from `Built`.
    ///
    /// Contracts are already compiled once without tests included to do `CONTRACT_ID` injection. `built_contracts` map holds already compiled contracts so that they can be matched with their "tests included" version.
    pub(crate) fn from_built(built: Built) -> anyhow::Result<BuiltTests> {
        let built = match built {
            Built::Package(built_pkg) => {
                BuiltTests::Package(PackageTests::from_built_pkg(*built_pkg))
            }
            Built::Workspace(built_workspace) => {
                let pkg_tests = built_workspace
                    .into_values()
                    .map(PackageTests::from_built_pkg)
                    .collect();
                BuiltTests::Workspace(pkg_tests)
            }
        };
        Ok(built)
    }
}

impl<'a> PackageTests {
    /// Return a reference to the underlying `BuiltPackage`.
    ///
    /// If this `PackageTests` is `PackageTests::Contract`, built package with tests included is
    /// returned.
    pub(crate) fn built_pkg_with_tests(&'a self) -> &'a BuiltPackage {
        match self {
            PackageTests::Contract(contract) => &contract.pkg,
            PackageTests::NonContract(non_contract) => non_contract,
        }
    }

    /// Construct a `PackageTests` from `BuiltPackage`.
    ///
    /// If the `BuiltPackage` is a contract, match the contract with the contract's
    fn from_built_pkg(built_pkg: BuiltPackage) -> PackageTests {
        let built_without_tests_bytecode = built_pkg.bytecode_without_tests.clone();
        match built_without_tests_bytecode {
            Some(contract_without_tests) => {
                let contract_to_test = ContractToTest {
                    pkg: built_pkg,
                    without_tests_bytecode: contract_without_tests,
                };
                PackageTests::Contract(contract_to_test)
            }
            None => PackageTests::NonContract(built_pkg),
        }
    }

    /// Run all tests for this package and collect their results.
    pub(crate) fn run_tests(&self) -> anyhow::Result<TestedPackage> {
        // TODO: Remove this once https://github.com/FuelLabs/sway/issues/3947 is solved.
        let mut visited_tests = HashSet::new();
        let pkg_with_tests = self.built_pkg_with_tests();
        // TODO: We can easily parallelise this, but let's wait until testing is stable first.
        let tests = pkg_with_tests
            .bytecode
            .entries
            .iter()
            .filter_map(|entry| entry.kind.test().map(|test| (entry, test)))
            .filter(|(_, test_entry)| visited_tests.insert(&test_entry.span))
            .map(|(entry, test_entry)| {
                let offset = u32::try_from(entry.finalized.imm)
                    .expect("test instruction offset out of range");
                let name = entry.finalized.fn_name.clone();
                let test_setup = self.setup()?;
                let (state, duration, receipts) =
                    exec_test(&pkg_with_tests.bytecode.bytes, offset, test_setup);

                let gas_used = *receipts
                    .iter()
                    .find_map(|receipt| match receipt {
                        tx::Receipt::ScriptResult { gas_used, .. } => Some(gas_used),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!("missing used gas information from test execution")
                    })?;

                // Only retain `Log` and `LogData` receipts.
                let logs = receipts
                    .into_iter()
                    .filter(|receipt| {
                        matches!(receipt, fuel_tx::Receipt::Log { .. })
                            || matches!(receipt, fuel_tx::Receipt::LogData { .. })
                    })
                    .collect();

                let span = test_entry.span.clone();
                let condition = test_entry.pass_condition.clone();
                Ok(TestResult {
                    name,
                    duration,
                    span,
                    state,
                    condition,
                    logs,
                    gas_used,
                })
            })
            .collect::<anyhow::Result<_>>()?;
        let tested_pkg = TestedPackage {
            built: Box::new(pkg_with_tests.clone()),
            tests,
        };
        Ok(tested_pkg)
    }

    /// Setup the storage for a test and returns a contract id for testing contracts.
    ///
    /// For testing contracts, storage returned from this function contains the deployed contract.
    /// For other types, default storage is returned.
    fn setup(&self) -> anyhow::Result<TestSetup> {
        match self {
            PackageTests::Contract(contract_to_test) => {
                let contract_pkg = &contract_to_test.pkg;
                let contract_pkg_without_tests = &contract_to_test.without_tests_bytecode;
                let test_setup = deploy_test_contract(contract_pkg, contract_pkg_without_tests)?;
                Ok(test_setup)
            }
            PackageTests::NonContract(_) => Ok(TestSetup {
                storage: vm::storage::MemoryStorage::default(),
                contract_id: None,
            }),
        }
    }
}

impl Opts {
    /// Convert this set of test options into a set of build options.
    pub fn into_build_opts(self) -> pkg::BuildOpts {
        let const_inject_map = std::collections::HashMap::new();
        pkg::BuildOpts {
            pkg: self.pkg,
            print: self.print,
            minify: self.minify,
            binary_outfile: self.binary_outfile,
            debug_outfile: self.debug_outfile,
            build_target: self.build_target,
            build_profile: self.build_profile,
            release: self.release,
            error_on_warnings: self.error_on_warnings,
            time_phases: self.time_phases,
            tests: true,
            const_inject_map,
            member_filter: Default::default(),
        }
    }
}

impl TestResult {
    /// Whether or not the test passed.
    pub fn passed(&self) -> bool {
        match &self.condition {
            TestPassCondition::ShouldRevert => {
                matches!(self.state, vm::state::ProgramState::Revert(_))
            }
            TestPassCondition::ShouldNotRevert => {
                !matches!(self.state, vm::state::ProgramState::Revert(_))
            }
        }
    }

    /// Return the revert code for this `TestResult` if the test is reverted.
    pub fn revert_code(&self) -> Option<u64> {
        match self.state {
            vm::state::ProgramState::Revert(revert_code) => Some(revert_code),
            _ => None,
        }
    }

    /// Return a `ErrorSignal` for this `TestResult` if the test is failed to pass.
    pub fn error_signal(&self) -> anyhow::Result<ErrorSignal> {
        let revert_code = self.revert_code().ok_or_else(|| {
            anyhow::anyhow!("there is no revert code to convert to `ErrorSignal`")
        })?;
        ErrorSignal::try_from_revert_code(revert_code).map_err(|e| anyhow::anyhow!(e))
    }

    /// Return `TestDetails` from the span of the function declaring this test.
    pub fn details(&self) -> anyhow::Result<TestDetails> {
        let file_path = self
            .span
            .path()
            .ok_or_else(|| anyhow::anyhow!("Missing span for test function"))?
            .to_owned();
        let span_start = self.span.start();
        let file_str = fs::read_to_string(&*file_path)?;
        let line_number = file_str[..span_start]
            .chars()
            .into_iter()
            .filter(|&c| c == '\n')
            .count();
        Ok(TestDetails {
            file_path,
            line_number,
        })
    }
}

impl BuiltTests {
    /// The total number of tests.
    pub fn test_count(&self) -> usize {
        // TODO: Remove this once https://github.com/FuelLabs/sway/issues/3947 is solved.
        let mut visited_tests = HashSet::new();

        let pkgs: Vec<&PackageTests> = match self {
            BuiltTests::Package(pkg) => vec![pkg],
            BuiltTests::Workspace(workspace) => workspace.iter().collect(),
        };
        pkgs.iter()
            .map(|pkg| {
                pkg.built_pkg_with_tests()
                    .bytecode
                    .entries
                    .iter()
                    .filter_map(|entry| entry.kind.test().map(|test| (entry, test)))
                    .filter(|(_, test_entry)| visited_tests.insert(&test_entry.span))
                    .count()
            })
            .sum()
    }

    /// Run all built tests, return the result.
    pub fn run(self) -> anyhow::Result<Tested> {
        run_tests(self)
    }
}

/// First builds the package or workspace, ready for execution.
pub fn build(opts: Opts) -> anyhow::Result<BuiltTests> {
    let build_opts = opts.into_build_opts();
    let built = pkg::build_with_options(build_opts)?;
    BuiltTests::from_built(built)
}

/// Deploys the provided contract and returns an interpreter instance ready to be used in test
/// executions with deployed contract.
fn deploy_test_contract(
    built_pkg: &pkg::BuiltPackage,
    without_tests_bytecode: &pkg::BuiltPackageBytecode,
) -> anyhow::Result<TestSetup> {
    // Obtain the contract id for deployment.
    let mut storage_slots = built_pkg.storage_slots.clone();
    storage_slots.sort();
    let bytecode = &without_tests_bytecode.bytes;
    let contract = tx::Contract::from(bytecode.clone());
    let root = contract.root();
    let state_root = tx::Contract::initial_state_root(storage_slots.iter());
    let salt = tx::Salt::zeroed();
    let contract_id = contract.id(&salt, &root, &state_root);

    // Setup the interpreter for deployment.
    let params = tx::ConsensusParameters::default();
    let storage = vm::storage::MemoryStorage::default();
    let mut interpreter =
        vm::interpreter::Interpreter::with_storage(storage, params, GasCosts::default());

    // Create the deployment transaction.
    let mut rng = rand::rngs::StdRng::seed_from_u64(TEST_METADATA_SEED);

    // Prepare the transaction metadata.
    let secret_key = rng.gen();
    let utxo_id = rng.gen();
    let amount = 1;
    let maturity = 1;
    let asset_id = rng.gen();
    let tx_pointer = rng.gen();
    let block_height = (u32::MAX >> 1) as u64;

    let tx = tx::TransactionBuilder::create(bytecode.as_slice().into(), salt, storage_slots)
        .add_unsigned_coin_input(secret_key, utxo_id, amount, asset_id, tx_pointer, maturity)
        .add_output(tx::Output::contract_created(contract_id, state_root))
        .maturity(maturity)
        .finalize_checked(block_height, &params, &GasCosts::default());

    // Deploy the contract.
    interpreter.transact(tx)?;
    let storage_after_deploy = interpreter.as_ref();
    Ok(TestSetup {
        storage: storage_after_deploy.clone(),
        contract_id: Some(contract_id),
    })
}

/// Build the given package and run its tests, returning the results.
fn run_tests(built: BuiltTests) -> anyhow::Result<Tested> {
    match built {
        BuiltTests::Package(pkg) => {
            let tested_pkg = pkg.run_tests()?;
            Ok(Tested::Package(Box::new(tested_pkg)))
        }
        BuiltTests::Workspace(workspace) => {
            let tested_pkgs = workspace
                .into_iter()
                .map(|pkg| pkg.run_tests())
                .collect::<anyhow::Result<Vec<TestedPackage>>>()?;
            Ok(Tested::Workspace(tested_pkgs))
        }
    }
}

/// Given some bytecode and an instruction offset for some test's desired entry point, patch the
/// bytecode with a `JI` (jump) instruction to jump to the desired test.
///
/// We want to splice in the `JI` only after the initial data section setup is complete, and only
/// if the entry point doesn't begin exactly after the data section setup.
///
/// The following is how the beginning of the bytecode is laid out:
///
/// ```ignore
/// [0] ji   i4                       ; Jumps to the data section setup.
/// [1] noop
/// [2] DATA_SECTION_OFFSET[0..32]
/// [3] DATA_SECTION_OFFSET[32..64]
/// [4] lw   $ds $is 1                ; The data section setup, i.e. where the first ji lands.
/// [5] add  $$ds $$ds $is
/// [6] <first-entry-point>           ; This is where we want to jump from to our test code!
/// ```
fn patch_test_bytecode(bytecode: &[u8], test_offset: u32) -> std::borrow::Cow<[u8]> {
    // TODO: Standardize this or add metadata to bytecode.
    const PROGRAM_START_INST_OFFSET: u32 = 6;
    const PROGRAM_START_BYTE_OFFSET: usize = PROGRAM_START_INST_OFFSET as usize * Instruction::SIZE;

    // If our desired entry point is the program start, no need to jump.
    if test_offset == PROGRAM_START_INST_OFFSET {
        return std::borrow::Cow::Borrowed(bytecode);
    }

    // Create the jump instruction and splice it into the bytecode.
    let ji = fuel_asm::op::ji(test_offset);
    let ji_bytes = ji.to_bytes();
    let start = PROGRAM_START_BYTE_OFFSET;
    let end = start + ji_bytes.len();
    let mut patched = bytecode.to_vec();
    patched.splice(start..end, ji_bytes);
    std::borrow::Cow::Owned(patched)
}

// Execute the test whose entry point is at the given instruction offset as if it were a script.
fn exec_test(
    bytecode: &[u8],
    test_offset: u32,
    test_setup: TestSetup,
) -> (
    vm::state::ProgramState,
    std::time::Duration,
    Vec<fuel_tx::Receipt>,
) {
    let storage = test_setup.storage;
    let contract_id = test_setup.contract_id;

    // Patch the bytecode to jump to the relevant test.
    let bytecode = patch_test_bytecode(bytecode, test_offset).into_owned();

    // Create a transaction to execute the test function.
    let script_input_data = vec![];
    let mut rng = rand::rngs::StdRng::seed_from_u64(TEST_METADATA_SEED);

    // Prepare the transaction metadata.
    let secret_key = rng.gen();
    let utxo_id = rng.gen();
    let amount = 1;
    let maturity = 1;
    let asset_id = rng.gen();
    let tx_pointer = rng.gen();
    let block_height = (u32::MAX >> 1) as u64;

    let params = tx::ConsensusParameters::default();
    let mut tx = tx::TransactionBuilder::script(bytecode, script_input_data)
        .add_unsigned_coin_input(secret_key, utxo_id, amount, asset_id, tx_pointer, 0)
        .gas_limit(tx::ConsensusParameters::DEFAULT.max_gas_per_tx)
        .maturity(maturity)
        .clone();
    if let Some(contract_id) = contract_id {
        tx.add_input(tx::Input::Contract {
            utxo_id: tx::UtxoId::new(tx::Bytes32::zeroed(), 0),
            balance_root: tx::Bytes32::zeroed(),
            state_root: tx::Bytes32::zeroed(),
            tx_pointer: tx::TxPointer::new(0, 0),
            contract_id,
        })
        .add_output(tx::Output::Contract {
            input_index: 1,
            balance_root: fuel_tx::Bytes32::zeroed(),
            state_root: tx::Bytes32::zeroed(),
        });
    }
    let tx = tx.finalize_checked(block_height, &params, &GasCosts::default());

    let mut interpreter =
        vm::interpreter::Interpreter::with_storage(storage, params, GasCosts::default());

    // Execute and return the result.
    let start = std::time::Instant::now();
    let transition = interpreter.transact(tx).unwrap();
    let duration = start.elapsed();
    let state = *transition.state();
    let receipts = transition.receipts().to_vec();

    (state, duration, receipts)
}
