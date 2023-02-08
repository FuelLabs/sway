use std::collections::HashSet;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use forc_pkg as pkg;
use fuel_tx as tx;
use fuel_vm::checked_transaction::builder::TransactionBuilderExt;
use fuel_vm::gas::GasCosts;
use fuel_vm::{self as vm, fuel_asm, prelude::Instruction};
use pkg::TestPassCondition;
use pkg::{Built, BuiltPackage, CONTRACT_ID_CONSTANT_NAME};
use rand::{Rng, SeedableRng};
use sway_core::{language::parsed::TreeType, BuildTarget};
use sway_types::{ConfigTimeConstant, Span};

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
    /// The span for the function declaring this tests.
    pub span: Span,
    /// The resulting state after executing the test function.
    pub state: vm::state::ProgramState,
    /// The required state of the VM for this test to pass.
    pub condition: pkg::TestPassCondition,
    /// Emitted `Recipt`s during the execution of the test.
    pub logs: Vec<fuel_tx::Receipt>,
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
///
/// `tests_included` is the built pkg with the `--test` flag, (i.e `forc build --tests`).
/// `tests_excluded` is the built pkg without the `--test` flag (i.e `forc build`).
#[derive(Debug)]
pub struct ContractToTest {
    pub tests_included: pkg::BuiltPackage,
    pub tests_excluded: pkg::BuiltPackage,
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
    pub(crate) fn from_built(
        built: Built,
        built_contracts: HashMap<pkg::Pinned, BuiltPackage>,
    ) -> anyhow::Result<BuiltTests> {
        let built = match built {
            Built::Package(built_pkg) => {
                BuiltTests::Package(PackageTests::from_built_pkg(*built_pkg, &built_contracts)?)
            }
            Built::Workspace(built_workspace) => {
                let pkg_tests = built_workspace
                    .into_values()
                    .map(|built_pkg| PackageTests::from_built_pkg(built_pkg, &built_contracts))
                    .collect::<anyhow::Result<_>>()?;
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
            PackageTests::Contract(contract) => &contract.tests_included,
            PackageTests::NonContract(non_contract) => non_contract,
        }
    }

    /// Construct a `PackageTests` from `BuiltPackage`.
    ///
    /// If the `BuiltPackage` is a contract, match the contract with the contract's
    fn from_built_pkg(
        built_pkg: BuiltPackage,
        built_contracts: &HashMap<pkg::Pinned, BuiltPackage>,
    ) -> anyhow::Result<PackageTests> {
        let tree_type = &built_pkg.tree_type;
        let package_test = match tree_type {
            sway_core::language::parsed::TreeType::Contract => {
                let built_pkg_descriptor = &built_pkg.built_pkg_descriptor;
                let built_contract_without_tests = built_contracts
                    .get(&built_pkg_descriptor.pinned)
                    .ok_or_else(|| anyhow::anyhow!("missing built contract without tests"))?;
                let contract_to_test = ContractToTest {
                    tests_included: built_pkg,
                    tests_excluded: built_contract_without_tests.clone(),
                };
                PackageTests::Contract(contract_to_test)
            }
            _ => PackageTests::NonContract(built_pkg),
        };
        Ok(package_test)
    }

    /// Run all tests for this package and collect their results.
    pub(crate) fn run_tests(&self) -> anyhow::Result<TestedPackage> {
        // TODO: Remove this once https://github.com/FuelLabs/sway/issues/3947 is solved.
        let mut visited_tests = HashSet::new();
        let pkg_with_tests = self.built_pkg_with_tests();
        // TODO: We can easily parallelise this, but let's wait until testing is stable first.
        let tests = pkg_with_tests
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
                    exec_test(&pkg_with_tests.bytecode, offset, test_setup);

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
                let contract_pkg_without_tests = contract_to_test.tests_excluded.clone();
                let test_setup = deploy_test_contract(contract_pkg_without_tests)?;
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
            time_phases: self.time_phases,
            tests: true,
            const_inject_map,
        }
    }

    /// Patch this set of test options, so that it will build the package at the given `path`.
    pub(crate) fn with_path(&mut self, path: &std::path::Path) -> Opts {
        self.pkg.path = path.to_str().map(ToString::to_string);
        self.clone()
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
        let pkgs: Vec<&PackageTests> = match self {
            BuiltTests::Package(pkg) => vec![pkg],
            BuiltTests::Workspace(workspace) => workspace.iter().collect(),
        };
        pkgs.iter()
            .map(|pkg| {
                pkg.built_pkg_with_tests()
                    .entries
                    .iter()
                    .filter(|e| e.is_test())
                    .count()
            })
            .sum()
    }

    /// Run all built tests, return the result.
    pub fn run(self) -> anyhow::Result<Tested> {
        run_tests(self)
    }
}

/// Build all contracts in the given buld plan without tests.
fn build_contracts_without_tests(
    opts: &Opts,
    build_plan: &pkg::BuildPlan,
) -> Vec<(pkg::Pinned, anyhow::Result<BuiltPackage>)> {
    let manifest_map = build_plan.manifest_map();
    build_plan
        .member_pinned_pkgs()
        .map(|pinned_pkg| {
            let pkg_manifest = manifest_map
                .get(&pinned_pkg.id())
                .expect("missing manifest for member to test");
            (pinned_pkg, pkg_manifest)
        })
        .filter(|(_, pkg_manifest)| matches!(pkg_manifest.program_type(), Ok(TreeType::Contract)))
        .map(|(pinned_pkg, pkg_manifest)| {
            let pkg_path = pkg_manifest.dir();
            let build_opts_without_tests = opts
                .clone()
                .with_path(pkg_path)
                .into_build_opts()
                .include_tests(false);
            let built_pkg =
                pkg::build_with_options(build_opts_without_tests).and_then(|pkg| pkg.expect_pkg());
            (pinned_pkg, built_pkg)
        })
        .collect()
}

/// First builds the package or workspace, ready for execution.
///
/// If the workspace contains contracts, those contracts will be built first without tests
/// in order to determine their `CONTRACT_ID`s and enable contract calling.
pub fn build(opts: Opts) -> anyhow::Result<BuiltTests> {
    let build_opts = opts.clone().into_build_opts();

    let build_plan = pkg::BuildPlan::from_build_opts(&build_opts)?;
    let mut const_inject_map = HashMap::new();
    let mut built_contracts = HashMap::new();
    let built_contracts_without_tests = build_contracts_without_tests(&opts, &build_plan);
    for (pinned_contract, built_contract) in built_contracts_without_tests {
        let built_contract = built_contract?;
        let contract_id = pkg::contract_id(&built_contract, &fuel_tx::Salt::zeroed());
        built_contracts.insert(pinned_contract.clone(), built_contract);

        // Construct namespace with contract id
        let contract_id_constant_name = CONTRACT_ID_CONSTANT_NAME.to_string();
        let contract_id_value = format!("0x{contract_id}");
        let contract_id_constant = ConfigTimeConstant {
            r#type: "b256".to_string(),
            value: contract_id_value.clone(),
            public: true,
        };
        let constant_declarations = vec![(contract_id_constant_name, contract_id_constant)];
        const_inject_map.insert(pinned_contract, constant_declarations);
    }

    // Injection map is collected in the previous pass, we should build the workspace/package with injection map.
    let build_opts_with_injection = build_opts.const_injection_map(const_inject_map);
    let built = pkg::build_with_options(build_opts_with_injection)?;
    BuiltTests::from_built(built, built_contracts)
}

/// Deploys the provided contract and returns an interpreter instance ready to be used in test
/// executions with deployed contract.
fn deploy_test_contract(built_pkg: BuiltPackage) -> anyhow::Result<TestSetup> {
    // Obtain the contract id for deployment.
    let mut storage_slots = built_pkg.storage_slots;
    storage_slots.sort();
    let bytecode = built_pkg.bytecode;
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

    let tx = tx::TransactionBuilder::create(bytecode.into(), salt, storage_slots)
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
