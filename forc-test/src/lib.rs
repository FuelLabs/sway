use forc_pkg as pkg;
use fuel_abi_types::error_codes::ErrorSignal;
use fuel_tx as tx;
use fuel_vm::checked_transaction::builder::TransactionBuilderExt;
use fuel_vm::gas::GasCosts;
use fuel_vm::{self as vm, fuel_asm, prelude::Instruction};
use pkg::TestPassCondition;
use pkg::{Built, BuiltPackage};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
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
/// A mapping from each member package of a build plan to its compiled contract dependencies.
type ContractDependencyMap = HashMap<pkg::Pinned, Vec<Arc<pkg::BuiltPackage>>>;

/// A package or a workspace that has been built, ready for test execution.
pub enum BuiltTests {
    Package(PackageTests),
    Workspace(Vec<PackageTests>),
}

/// A built package ready for test execution.
///
/// If the built package is a contract, a second built package for the same contract without the
/// tests are also populated.
///
/// For packages containing contracts or scripts, their [contract-dependencies] are needed for deployment.
#[derive(Debug)]
pub enum PackageTests {
    Contract(PackageWithDeploymentToTest),
    Script(PackageWithDeploymentToTest),
    Predicate(Arc<pkg::BuiltPackage>),
    Library(Arc<pkg::BuiltPackage>),
}

/// A built contract ready for test execution.
#[derive(Debug)]
pub struct ContractToTest {
    /// Tests included contract.
    pkg: Arc<pkg::BuiltPackage>,
    /// Bytecode of the contract without tests.
    without_tests_bytecode: pkg::BuiltPackageBytecode,
    contract_dependencies: Vec<Arc<pkg::BuiltPackage>>,
}

/// A built script ready for test execution.
#[derive(Debug)]
pub struct ScriptToTest {
    /// Tests included contract.
    pkg: Arc<pkg::BuiltPackage>,
    contract_dependencies: Vec<Arc<pkg::BuiltPackage>>,
}

/// A built package that requires deployment before test execution.
#[derive(Debug)]
pub enum PackageWithDeploymentToTest {
    Script(ScriptToTest),
    Contract(ContractToTest),
}

/// Required test setup for package types that requires a deployment.
#[derive(Debug)]
enum DeploymentSetup {
    Script(ScriptTestSetup),
    Contract(ContractTestSetup),
}

impl DeploymentSetup {
    /// Returns the storage for this test setup
    fn storage(&self) -> &vm::storage::MemoryStorage {
        match self {
            DeploymentSetup::Script(script_setup) => &script_setup.storage,
            DeploymentSetup::Contract(contract_setup) => &contract_setup.storage,
        }
    }

    /// Return the root contract id if this is a contract setup.
    fn root_contract_id(&self) -> Option<tx::ContractId> {
        match self {
            DeploymentSetup::Script(_) => None,
            DeploymentSetup::Contract(contract_setup) => Some(contract_setup.root_contract_id),
        }
    }
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
    /// Enable the experimental storage implementation and UI.
    pub experimental_storage: bool,
}

/// The set of options provided for controlling logs printed for each test.
#[derive(Default, Clone)]
pub struct TestPrintOpts {
    pub pretty_print: bool,
    pub print_logs: bool,
}

/// The storage and the contract id (if a contract is being tested) for a test.
#[derive(Debug)]
enum TestSetup {
    WithDeployment(DeploymentSetup),
    WithoutDeployment(vm::storage::MemoryStorage),
}

impl TestSetup {
    /// Returns the storage for this test setup
    fn storage(&self) -> &vm::storage::MemoryStorage {
        match self {
            TestSetup::WithDeployment(deployment_setup) => deployment_setup.storage(),
            TestSetup::WithoutDeployment(storage) => storage,
        }
    }

    /// Produces an iterator yielding contract ids of contract dependencies for this test setup.
    fn contract_dependency_ids(&self) -> impl Iterator<Item = &tx::ContractId> + '_ {
        match self {
            TestSetup::WithDeployment(deployment_setup) => match deployment_setup {
                DeploymentSetup::Script(script_setup) => {
                    script_setup.contract_dependency_ids.iter()
                }
                DeploymentSetup::Contract(contract_setup) => {
                    contract_setup.contract_dependency_ids.iter()
                }
            },
            TestSetup::WithoutDeployment(_) => [].iter(),
        }
    }

    /// Return the root contract id if this is a contract setup.
    fn root_contract_id(&self) -> Option<tx::ContractId> {
        match self {
            TestSetup::WithDeployment(deployment_setup) => deployment_setup.root_contract_id(),
            TestSetup::WithoutDeployment(_) => None,
        }
    }

    /// Produces an iterator yielding all contract ids required to be included in the transaction
    /// for this test setup.
    fn contract_ids(&self) -> impl Iterator<Item = tx::ContractId> + '_ {
        self.contract_dependency_ids()
            .cloned()
            .chain(self.root_contract_id())
    }
}

/// The data collected to test a contract.
#[derive(Debug)]
struct ContractTestSetup {
    storage: vm::storage::MemoryStorage,
    contract_dependency_ids: Vec<tx::ContractId>,
    root_contract_id: tx::ContractId,
}

/// The data collected to test a script.
#[derive(Debug)]
struct ScriptTestSetup {
    storage: vm::storage::MemoryStorage,
    contract_dependency_ids: Vec<tx::ContractId>,
}

impl TestedPackage {
    pub fn tests_passed(&self) -> bool {
        self.tests.iter().all(|test| test.passed())
    }
}

impl PackageWithDeploymentToTest {
    /// Returns a reference to the underlying `BuiltPackage`.
    ///
    /// If this is a contract built package with tests included is returned.
    fn pkg(&self) -> &BuiltPackage {
        match self {
            PackageWithDeploymentToTest::Script(script) => &script.pkg,
            PackageWithDeploymentToTest::Contract(contract) => &contract.pkg,
        }
    }

    /// Returns an iterator over contract dependencies of the package represented by this struct.
    fn contract_dependencies(&self) -> impl Iterator<Item = &Arc<BuiltPackage>> + '_ {
        match self {
            PackageWithDeploymentToTest::Script(script_to_test) => {
                script_to_test.contract_dependencies.iter()
            }
            PackageWithDeploymentToTest::Contract(contract_to_test) => {
                contract_to_test.contract_dependencies.iter()
            }
        }
    }

    /// Deploy the contract dependencies for packages that require deployment.
    ///
    /// For scripts deploys all contract dependencies.
    /// For contract deploys all contract dependencies and the root contract itself.
    fn deploy(&self) -> anyhow::Result<TestSetup> {
        // Setup the interpreter for deployment.
        let params = tx::ConsensusParameters::default();
        let storage = vm::storage::MemoryStorage::default();
        let mut interpreter =
            vm::interpreter::Interpreter::with_storage(storage, params, GasCosts::default());

        // Iterate and create deployment transactions for contract dependencies of the root
        // contract.
        let contract_dependency_setups = self
            .contract_dependencies()
            .map(|built_pkg| deployment_transaction(built_pkg, &built_pkg.bytecode, params));

        // Deploy contract dependencies of the root contract and collect their ids.
        let contract_dependency_ids = contract_dependency_setups
            .map(|(contract_id, tx)| {
                // Transact the deployment transaction constructed for this contract dependency.
                interpreter.transact(tx)?;
                Ok(contract_id)
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let deployment_setup = if let PackageWithDeploymentToTest::Contract(contract_to_test) = self
        {
            // Root contract is the contract that we are going to be running the tests of, after this
            // deployment.
            let (root_contract_id, root_contract_tx) = deployment_transaction(
                &contract_to_test.pkg,
                &contract_to_test.without_tests_bytecode,
                params,
            );
            // Deploy the root contract.
            interpreter.transact(root_contract_tx)?;
            let storage = interpreter.as_ref().clone();
            DeploymentSetup::Contract(ContractTestSetup {
                storage,
                contract_dependency_ids,
                root_contract_id,
            })
        } else {
            let storage = interpreter.as_ref().clone();
            DeploymentSetup::Script(ScriptTestSetup {
                storage,
                contract_dependency_ids,
            })
        };

        Ok(TestSetup::WithDeployment(deployment_setup))
    }
}

impl BuiltTests {
    /// Constructs a `PackageTests` from `Built`.
    ///
    /// `contract_dependencies` represents ordered (by deployment order) packages that needs to be deployed for each package, before executing the test.
    pub(crate) fn from_built(
        built: Built,
        contract_dependencies: &ContractDependencyMap,
    ) -> anyhow::Result<BuiltTests> {
        let built = match built {
            Built::Package(built_pkg) => BuiltTests::Package(PackageTests::from_built_pkg(
                built_pkg,
                contract_dependencies,
            )),
            Built::Workspace(built_workspace) => {
                let pkg_tests = built_workspace
                    .into_iter()
                    .map(|built_pkg| PackageTests::from_built_pkg(built_pkg, contract_dependencies))
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
            PackageTests::Contract(contract) => contract.pkg(),
            PackageTests::Script(script) => script.pkg(),
            PackageTests::Predicate(predicate) => predicate,
            PackageTests::Library(library) => library,
        }
    }

    /// Construct a `PackageTests` from `BuiltPackage`.
    fn from_built_pkg(
        built_pkg: Arc<BuiltPackage>,
        contract_dependencies: &ContractDependencyMap,
    ) -> PackageTests {
        let built_without_tests_bytecode = built_pkg.bytecode_without_tests.clone();
        let contract_dependencies: Vec<Arc<pkg::BuiltPackage>> = contract_dependencies
            .get(&built_pkg.descriptor.pinned)
            .cloned()
            .unwrap_or_default();
        match built_without_tests_bytecode {
            Some(contract_without_tests) => {
                let contract_to_test = ContractToTest {
                    pkg: built_pkg,
                    without_tests_bytecode: contract_without_tests,
                    contract_dependencies,
                };
                PackageTests::Contract(PackageWithDeploymentToTest::Contract(contract_to_test))
            }
            None => match built_pkg.tree_type {
                sway_core::language::parsed::TreeType::Predicate => {
                    PackageTests::Predicate(built_pkg)
                }
                sway_core::language::parsed::TreeType::Library => PackageTests::Library(built_pkg),
                sway_core::language::parsed::TreeType::Script => {
                    let script_to_test = ScriptToTest {
                        pkg: built_pkg,
                        contract_dependencies,
                    };
                    PackageTests::Script(PackageWithDeploymentToTest::Script(script_to_test))
                }
                _ => unreachable!("contracts are already handled"),
            },
        }
    }

    /// Run all tests for this package and collect their results.
    pub(crate) fn run_tests(
        &self,
        test_runners: &rayon::ThreadPool,
    ) -> anyhow::Result<TestedPackage> {
        let pkg_with_tests = self.built_pkg_with_tests();
        let tests = test_runners.install(|| {
            pkg_with_tests
                .bytecode
                .entries
                .par_iter()
                .filter_map(|entry| entry.kind.test().map(|test| (entry, test)))
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
                .collect::<anyhow::Result<_>>()
        })?;

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
                let test_setup = contract_to_test.deploy()?;
                Ok(test_setup)
            }
            PackageTests::Script(script_to_test) => {
                let test_setup = script_to_test.deploy()?;
                Ok(test_setup)
            }
            PackageTests::Predicate(_) | PackageTests::Library(_) => Ok(
                TestSetup::WithoutDeployment(vm::storage::MemoryStorage::default()),
            ),
        }
    }
}

impl Opts {
    /// Convert this set of test options into a set of build options.
    pub fn into_build_opts(self) -> pkg::BuildOpts {
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
            member_filter: Default::default(),
            experimental_storage: self.experimental_storage,
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
            .filter(|&c| c == '\n')
            .count();
        Ok(TestDetails {
            file_path,
            line_number,
        })
    }
}

/// Used to control test runner count for forc-test. Number of runners to use can be specified using
/// `Manual` or can be left forc-test to decide by using `Auto`.
pub enum TestRunnerCount {
    Manual(usize),
    Auto,
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
                    .bytecode
                    .entries
                    .iter()
                    .filter_map(|entry| entry.kind.test().map(|test| (entry, test)))
                    .count()
            })
            .sum()
    }

    /// Run all built tests, return the result.
    pub fn run(self, test_runner_count: TestRunnerCount) -> anyhow::Result<Tested> {
        let test_runners = match test_runner_count {
            TestRunnerCount::Manual(runner_count) => rayon::ThreadPoolBuilder::new()
                .num_threads(runner_count)
                .build(),
            TestRunnerCount::Auto => rayon::ThreadPoolBuilder::new().build(),
        }?;
        run_tests(self, &test_runners)
    }
}

/// First builds the package or workspace, ready for execution.
pub fn build(opts: Opts) -> anyhow::Result<BuiltTests> {
    let build_opts = opts.into_build_opts();
    let build_plan = pkg::BuildPlan::from_build_opts(&build_opts)?;
    let built = pkg::build_with_options(build_opts)?;
    let built_members: HashMap<&pkg::Pinned, Arc<BuiltPackage>> = built.into_members().collect();

    // For each member node collect their contract dependencies.
    let member_contract_dependencies: HashMap<pkg::Pinned, Vec<Arc<pkg::BuiltPackage>>> =
        build_plan
            .member_nodes()
            .map(|member_node| {
                let graph = build_plan.graph();
                let pinned_member = graph[member_node].clone();
                let contract_dependencies = build_plan
                    .contract_dependencies(member_node)
                    .map(|contract_depency_node_ix| graph[contract_depency_node_ix].clone())
                    .filter_map(|pinned| built_members.get(&pinned))
                    .cloned()
                    .collect();

                (pinned_member, contract_dependencies)
            })
            .collect();
    BuiltTests::from_built(built, &member_contract_dependencies)
}

/// Result of preparing a deployment transaction setup for a contract.
type ContractDeploymentSetup = (tx::ContractId, vm::checked_transaction::Checked<tx::Create>);

/// Deploys the provided contract and returns an interpreter instance ready to be used in test
/// executions with deployed contract.
fn deployment_transaction(
    built_pkg: &pkg::BuiltPackage,
    without_tests_bytecode: &pkg::BuiltPackageBytecode,
    params: tx::ConsensusParameters,
) -> ContractDeploymentSetup {
    // Obtain the contract id for deployment.
    let mut storage_slots = built_pkg.storage_slots.clone();
    storage_slots.sort();
    let bytecode = &without_tests_bytecode.bytes;
    let contract = tx::Contract::from(bytecode.clone());
    let root = contract.root();
    let state_root = tx::Contract::initial_state_root(storage_slots.iter());
    let salt = tx::Salt::zeroed();
    let contract_id = contract.id(&salt, &root, &state_root);

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
    (contract_id, tx)
}

/// Build the given package and run its tests, returning the results.
fn run_tests(built: BuiltTests, test_runners: &rayon::ThreadPool) -> anyhow::Result<Tested> {
    match built {
        BuiltTests::Package(pkg) => {
            let tested_pkg = pkg.run_tests(test_runners)?;
            Ok(Tested::Package(Box::new(tested_pkg)))
        }
        BuiltTests::Workspace(workspace) => {
            let tested_pkgs = workspace
                .into_iter()
                .map(|pkg| pkg.run_tests(test_runners))
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
    let storage = test_setup.storage().clone();

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
    let mut output_index = 1;
    // Insert contract ids into tx input
    for contract_id in test_setup.contract_ids() {
        tx.add_input(tx::Input::Contract {
            utxo_id: tx::UtxoId::new(tx::Bytes32::zeroed(), 0),
            balance_root: tx::Bytes32::zeroed(),
            state_root: tx::Bytes32::zeroed(),
            tx_pointer: tx::TxPointer::new(0, 0),
            contract_id,
        })
        .add_output(tx::Output::Contract {
            input_index: output_index,
            balance_root: fuel_tx::Bytes32::zeroed(),
            state_root: tx::Bytes32::zeroed(),
        });
        output_index += 1;
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
