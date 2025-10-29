pub mod ecal;
pub mod execute;
pub mod setup;

use crate::execute::TestExecutor;
use crate::setup::{
    ContractDeploymentSetup, ContractTestSetup, DeploymentSetup, ScriptTestSetup, TestSetup,
};
use ecal::EcalSyscallHandler;
use forc_pkg::{self as pkg, BuildOpts, DumpOpts};
use forc_util::tx_utils::decode_fuel_vm_log_data;
use fuel_abi_types::abi::program::ProgramABI;
use fuel_abi_types::revert_info::RevertInfo;
use fuel_tx::{self as tx, GasCostsValues};
use fuel_vm::checked_transaction::builder::TransactionBuilderExt;
use fuel_vm::{self as vm};
use pkg::TestPassCondition;
use pkg::{Built, BuiltPackage};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::str::FromStr;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use sway_core::BuildTarget;
use sway_types::Span;
use tx::consensus_parameters::ConsensusParametersV1;
use tx::{ConsensusParameters, ContractParameters, ScriptParameters, TxParameters};
use vm::interpreter::{InterpreterParams, MemoryInstance};
use vm::prelude::SecretKey;

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

/// The filter to be used to only run matching tests.
#[derive(Debug, Clone)]
pub struct TestFilter<'a> {
    /// The phrase used for filtering, a `&str` searched/matched with test name.
    pub filter_phrase: &'a str,
    /// If set `true`, a complete "match" is required with test name for the test to be executed,
    /// otherwise a test_name should "contain" the `filter_phrase`.
    pub exact_match: bool,
}

/// The result of executing a single test within a single package.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// The name of the function.
    pub name: String,
    /// The time taken for the test to execute.
    pub duration: std::time::Duration,
    /// The span for the function declaring this test.
    pub span: Span,
    /// The file path for the function declaring this test.
    pub file_path: Arc<PathBuf>,
    /// The resulting state after executing the test function.
    pub state: vm::state::ProgramState,
    /// The required state of the VM for this test to pass.
    pub condition: pkg::TestPassCondition,
    /// Emitted `Receipt`s during the execution of the test.
    pub logs: Vec<fuel_tx::Receipt>,
    /// Gas used while executing this test.
    pub gas_used: u64,
    /// EcalState of the execution
    pub ecal: Box<EcalSyscallHandler>,
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

/// The set of options provided to the `test` function.
#[derive(Default, Clone)]
pub struct TestOpts {
    pub pkg: pkg::PkgOpts,
    pub print: pkg::PrintOpts,
    pub minify: pkg::MinifyOpts,
    /// If set, outputs a binary file representing the script bytes.
    pub binary_outfile: Option<String>,
    /// If set, outputs debug info to the provided file.
    /// If the argument provided ends with .json, a JSON is emitted,
    /// otherwise, an ELF file containing DWARF is emitted.
    pub debug_outfile: Option<String>,
    /// If set, generates a JSON file containing the hex-encoded script binary.
    pub hex_outfile: Option<String>,
    /// Build target to use.
    pub build_target: BuildTarget,
    /// Name of the build profile to use.
    pub build_profile: String,
    /// Use the release build profile.
    /// The release profile can be customized in the manifest file.
    pub release: bool,
    /// Should warnings be treated as errors?
    pub error_on_warnings: bool,
    /// Output the time elapsed over each part of the compilation process.
    pub time_phases: bool,
    /// Profile the compilation process.
    pub profile: bool,
    /// Output compilation metrics into file.
    pub metrics_outfile: Option<String>,
    /// Set of enabled experimental flags
    pub experimental: Vec<sway_features::Feature>,
    /// Set of disabled experimental flags
    pub no_experimental: Vec<sway_features::Feature>,
}

/// The set of options provided for controlling logs printed for each test.
#[derive(Default, Clone)]
pub struct TestPrintOpts {
    pub pretty_print: bool,
    pub print_logs: bool,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GasCostsSource {
    #[default]
    BuiltIn,
    Mainnet,
    Testnet,
    File(String),
}

impl GasCostsSource {
    pub fn provide_gas_costs(&self) -> Result<GasCostsValues, anyhow::Error> {
        match self {
            // Values in the `gas_costs_values.json` are taken from the `chain-configuration` repository:
            //      chain-configuration/upgradelog/ignition/consensus_parameters/6.json
            // Update these values when there are changes to the gas costs on-chain.
            Self::BuiltIn => Ok(serde_json::from_str(include_str!(
                "../gas_costs_values.json"
            ))?),
            // TODO: (GAS-COSTS) Fetch actual gas costs from mainnet/testnet and JSON file.
            //       See: https://github.com/FuelLabs/sway/issues/7472
            Self::Mainnet => Err(anyhow::anyhow!(
                "Fetching gas costs from mainnet is currently not implemented."
            )),
            Self::Testnet => Err(anyhow::anyhow!(
                "Fetching gas costs from testnet is currently not implemented."
            )),
            Self::File(_file_path) => Err(anyhow::anyhow!(
                "Loading gas costs from a JSON file is currently not implemented."
            )),
        }
    }
}

impl FromStr for GasCostsSource {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "built-in" => Ok(Self::BuiltIn),
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            file_path => Ok(Self::File(file_path.to_string())),
        }
    }
}

/// A `LogData` decoded into a human readable format with its type information.
pub struct DecodedLog {
    pub value: String,
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
        let gas_price = 0;
        // We are not concerned about gas costs of contract deployments for tests,
        // only the gas costs of test executions. So, we can simply provide the
        // default, built-in, gas costs values here.
        let params = maxed_consensus_params(GasCostsValues::default());
        let storage = vm::storage::MemoryStorage::default();
        let interpreter_params = InterpreterParams::new(gas_price, params.clone());
        let mut interpreter: vm::prelude::Interpreter<_, _, _, vm::interpreter::NotSupportedEcal> =
            vm::interpreter::Interpreter::with_storage(
                MemoryInstance::new(),
                storage,
                interpreter_params,
            );

        // Iterate and create deployment transactions for contract dependencies of the root
        // contract.
        let contract_dependency_setups = self
            .contract_dependencies()
            .map(|built_pkg| deployment_transaction(built_pkg, &built_pkg.bytecode, &params));

        // Deploy contract dependencies of the root contract and collect their ids.
        let contract_dependency_ids = contract_dependency_setups
            .map(|(contract_id, tx)| {
                // Transact the deployment transaction constructed for this contract dependency.
                let tx = tx
                    .into_ready(gas_price, params.gas_costs(), params.fee_params(), None)
                    .unwrap();
                interpreter.transact(tx).map_err(anyhow::Error::msg)?;
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
                &params,
            );
            let root_contract_tx = root_contract_tx
                .into_ready(gas_price, params.gas_costs(), params.fee_params(), None)
                .unwrap();
            // Deploy the root contract.
            interpreter
                .transact(root_contract_tx)
                .map_err(anyhow::Error::msg)?;
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

/// Returns a mapping of each member package of a build plan to its compiled contract dependencies,
/// ordered by deployment order.
///
/// Each dependency package needs to be deployed before executing the test for that package.
fn get_contract_dependency_map(
    built: &Built,
    build_plan: &pkg::BuildPlan,
) -> ContractDependencyMap {
    let built_members: HashMap<&pkg::Pinned, Arc<pkg::BuiltPackage>> =
        built.into_members().collect();
    // For each member node, collect their contract dependencies.
    build_plan
        .member_nodes()
        .map(|member_node| {
            let graph = build_plan.graph();
            let pinned_member = graph[member_node].clone();
            let contract_dependencies = build_plan
                .contract_dependencies(member_node)
                .map(|contract_dependency_node_ix| graph[contract_dependency_node_ix].clone())
                .filter_map(|pinned| built_members.get(&pinned))
                .cloned()
                .collect::<Vec<_>>();
            (pinned_member, contract_dependencies)
        })
        .collect()
}

impl BuiltTests {
    /// Constructs a `PackageTests` from `Built`.
    pub fn from_built(built: Built, build_plan: &pkg::BuildPlan) -> anyhow::Result<BuiltTests> {
        let contract_dependencies = get_contract_dependency_map(&built, build_plan);
        let built = match built {
            Built::Package(built_pkg) => BuiltTests::Package(PackageTests::from_built_pkg(
                built_pkg,
                &contract_dependencies,
            )),
            Built::Workspace(built_workspace) => {
                let pkg_tests = built_workspace
                    .into_iter()
                    .map(|built_pkg| {
                        PackageTests::from_built_pkg(built_pkg, &contract_dependencies)
                    })
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

    /// Run all tests after applying the provided filter and collect their results.
    pub(crate) fn run_tests(
        &self,
        test_runners: &rayon::ThreadPool,
        test_filter: Option<&TestFilter>,
        gas_costs_values: GasCostsValues,
    ) -> anyhow::Result<TestedPackage> {
        let pkg_with_tests = self.built_pkg_with_tests();
        let tests = test_runners.install(|| {
            pkg_with_tests
                .bytecode
                .entries
                .par_iter()
                .filter_map(|entry| {
                    if let Some(test_entry) = entry.kind.test() {
                        // If a test filter is specified, only the tests containing the filter phrase in
                        // their name are going to be executed.
                        let name = entry.finalized.fn_name.clone();
                        if let Some(filter) = test_filter {
                            if !filter.filter(&name) {
                                return None;
                            }
                        }
                        return Some((entry, test_entry));
                    }
                    None
                })
                .map(|(entry, test_entry)| {
                    // Execute the test and return the result.
                    let offset = u32::try_from(entry.finalized.imm)
                        .expect("test instruction offset out of range");
                    let name = entry.finalized.fn_name.clone();
                    let test_setup = self.setup()?;
                    TestExecutor::build(
                        &pkg_with_tests.bytecode.bytes,
                        offset,
                        test_setup,
                        test_entry,
                        name,
                        gas_costs_values.clone(),
                    )?
                    .execute()
                })
                .collect::<anyhow::Result<_>>()
        })?;

        Ok(TestedPackage {
            built: Box::new(pkg_with_tests.clone()),
            tests,
        })
    }

    /// Setup the storage for a test and returns a contract id for testing contracts.
    ///
    /// For testing contracts, storage returned from this function contains the deployed contract.
    /// For other types, default storage is returned.
    pub fn setup(&self) -> anyhow::Result<TestSetup> {
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

impl From<TestOpts> for pkg::BuildOpts {
    fn from(val: TestOpts) -> Self {
        pkg::BuildOpts {
            pkg: val.pkg,
            print: val.print,
            minify: val.minify,
            dump: DumpOpts::default(),
            binary_outfile: val.binary_outfile,
            debug_outfile: val.debug_outfile,
            hex_outfile: val.hex_outfile,
            build_target: val.build_target,
            build_profile: val.build_profile,
            release: val.release,
            error_on_warnings: val.error_on_warnings,
            time_phases: val.time_phases,
            profile: val.profile,
            metrics_outfile: val.metrics_outfile,
            tests: true,
            member_filter: Default::default(),
            experimental: val.experimental,
            no_experimental: val.no_experimental,
        }
    }
}

impl TestOpts {
    /// Convert this set of test options into a set of build options.
    pub fn into_build_opts(self) -> pkg::BuildOpts {
        pkg::BuildOpts {
            pkg: self.pkg,
            print: self.print,
            minify: self.minify,
            dump: DumpOpts::default(),
            binary_outfile: self.binary_outfile,
            debug_outfile: self.debug_outfile,
            hex_outfile: self.hex_outfile,
            build_target: self.build_target,
            build_profile: self.build_profile,
            release: self.release,
            error_on_warnings: self.error_on_warnings,
            time_phases: self.time_phases,
            profile: self.profile,
            metrics_outfile: self.metrics_outfile,
            tests: true,
            member_filter: Default::default(),
            experimental: self.experimental,
            no_experimental: self.no_experimental,
        }
    }
}

impl TestResult {
    /// Whether or not the test passed.
    pub fn passed(&self) -> bool {
        match &self.condition {
            TestPassCondition::ShouldRevert(revert_code) => match revert_code {
                Some(revert_code) => self.state == vm::state::ProgramState::Revert(*revert_code),
                None => matches!(self.state, vm::state::ProgramState::Revert(_)),
            },
            TestPassCondition::ShouldNotRevert => {
                !matches!(self.state, vm::state::ProgramState::Revert(_))
            }
        }
    }

    /// Return the revert code for this [TestResult] if the test is reverted.
    pub fn revert_code(&self) -> Option<u64> {
        match self.state {
            vm::state::ProgramState::Revert(revert_code) => Some(revert_code),
            _ => None,
        }
    }

    pub fn revert_info(
        &self,
        program_abi: Option<&ProgramABI>,
        logs: &[fuel_tx::Receipt],
    ) -> Option<RevertInfo> {
        let decode_last_log_data = |log_id: &str, program_abi: &ProgramABI| {
            logs.last()
                .and_then(|log| {
                    if let fuel_tx::Receipt::LogData {
                        data: Some(data), ..
                    } = log
                    {
                        decode_fuel_vm_log_data(log_id, data, program_abi).ok()
                    } else {
                        None
                    }
                })
                .map(|decoded_log| decoded_log.value)
        };

        self.revert_code()
            .map(|revert_code| RevertInfo::new(revert_code, program_abi, decode_last_log_data))
    }

    /// Return [TestDetails] from the span of the function declaring this test.
    pub fn details(&self) -> anyhow::Result<TestDetails> {
        let span_start = self.span.start();
        let file_str = fs::read_to_string(&*self.file_path)?;
        let line_number = file_str[..span_start]
            .chars()
            .filter(|&c| c == '\n')
            .count();
        Ok(TestDetails {
            file_path: self.file_path.clone(),
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

#[derive(Clone, Debug, Default)]
pub struct TestCount {
    pub total: usize,
    pub ignored: usize,
}

impl TestFilter<'_> {
    fn filter(&self, fn_name: &str) -> bool {
        if self.exact_match {
            fn_name == self.filter_phrase
        } else {
            fn_name.contains(self.filter_phrase)
        }
    }
}

impl BuiltTests {
    /// The total number of tests.
    pub fn test_count(&self, test_filter: Option<&TestFilter>) -> TestCount {
        let pkgs: Vec<&PackageTests> = match self {
            BuiltTests::Package(pkg) => vec![pkg],
            BuiltTests::Workspace(workspace) => workspace.iter().collect(),
        };
        pkgs.iter()
            .flat_map(|pkg| {
                pkg.built_pkg_with_tests()
                    .bytecode
                    .entries
                    .iter()
                    .filter_map(|entry| entry.kind.test().map(|test| (entry, test)))
            })
            .fold(TestCount::default(), |acc, (pkg_entry, _)| {
                let num_ignored = match &test_filter {
                    Some(filter) => {
                        if filter.filter(&pkg_entry.finalized.fn_name) {
                            acc.ignored
                        } else {
                            acc.ignored + 1
                        }
                    }
                    None => acc.ignored,
                };
                TestCount {
                    total: acc.total + 1,
                    ignored: num_ignored,
                }
            })
    }

    /// Run all built tests, return the result.
    pub fn run(
        self,
        test_runner_count: TestRunnerCount,
        test_filter: Option<TestFilter>,
        gas_costs_values: GasCostsValues,
    ) -> anyhow::Result<Tested> {
        let test_runners = match test_runner_count {
            TestRunnerCount::Manual(runner_count) => rayon::ThreadPoolBuilder::new()
                .num_threads(runner_count)
                .build(),
            TestRunnerCount::Auto => rayon::ThreadPoolBuilder::new().build(),
        }?;
        run_tests(self, &test_runners, test_filter, gas_costs_values)
    }
}

/// First builds the package or workspace, ready for execution.
pub fn build(opts: TestOpts) -> anyhow::Result<BuiltTests> {
    let build_opts: BuildOpts = opts.into();
    let build_plan = pkg::BuildPlan::from_pkg_opts(&build_opts.pkg)?;
    let built = pkg::build_with_options(&build_opts, None)?;
    BuiltTests::from_built(built, &build_plan)
}

/// Returns a `ConsensusParameters` which has maximum length/size allowance for scripts, contracts,
/// and transactions.
pub(crate) fn maxed_consensus_params(gas_costs_values: GasCostsValues) -> ConsensusParameters {
    let script_params = ScriptParameters::DEFAULT
        .with_max_script_length(u64::MAX)
        .with_max_script_data_length(u64::MAX);
    let tx_params = TxParameters::DEFAULT.with_max_size(u64::MAX);
    let contract_params = ContractParameters::DEFAULT
        .with_contract_max_size(u64::MAX)
        .with_max_storage_slots(u64::MAX);
    ConsensusParameters::V1(ConsensusParametersV1 {
        script_params,
        tx_params,
        contract_params,
        gas_costs: gas_costs_values.into(),
        ..Default::default()
    })
}

/// Deploys the provided contract and returns an interpreter instance ready to be used in test
/// executions with deployed contract.
fn deployment_transaction(
    built_pkg: &pkg::BuiltPackage,
    without_tests_bytecode: &pkg::BuiltPackageBytecode,
    params: &tx::ConsensusParameters,
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
    let rng = &mut rand::rngs::StdRng::seed_from_u64(TEST_METADATA_SEED);

    // Prepare the transaction metadata.
    let secret_key = SecretKey::random(rng);
    let utxo_id = rng.r#gen();
    let amount = 1;
    let maturity = 1u32.into();
    // NOTE: fuel-core is using dynamic asset id and interacting with the fuel-core, using static
    // asset id is not correct. But since forc-test maintains its own interpreter instance, correct
    // base asset id is indeed the static `tx::AssetId::BASE`.
    let asset_id = tx::AssetId::BASE;
    let tx_pointer = rng.r#gen();
    let block_height = (u32::MAX >> 1).into();

    let tx = tx::TransactionBuilder::create(bytecode.as_slice().into(), salt, storage_slots)
        .with_params(params.clone())
        .add_unsigned_coin_input(secret_key, utxo_id, amount, asset_id, tx_pointer)
        .add_output(tx::Output::contract_created(contract_id, state_root))
        .maturity(maturity)
        .finalize_checked(block_height);
    (contract_id, tx)
}

/// Build the given package and run its tests after applying the filter provided.
///
/// Returns the result of test execution.
fn run_tests(
    built: BuiltTests,
    test_runners: &rayon::ThreadPool,
    test_filter: Option<TestFilter>,
    gas_costs_values: GasCostsValues,
) -> anyhow::Result<Tested> {
    match built {
        BuiltTests::Package(pkg) => {
            let tested_pkg =
                pkg.run_tests(test_runners, test_filter.as_ref(), gas_costs_values.clone())?;
            Ok(Tested::Package(Box::new(tested_pkg)))
        }
        BuiltTests::Workspace(workspace) => {
            let tested_pkgs = workspace
                .into_iter()
                .map(|pkg| {
                    pkg.run_tests(test_runners, test_filter.as_ref(), gas_costs_values.clone())
                })
                .collect::<anyhow::Result<Vec<TestedPackage>>>()?;
            Ok(Tested::Workspace(tested_pkgs))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fuel_tx::GasCostsValues;

    use crate::{build, BuiltTests, TestFilter, TestOpts, TestResult};

    /// Name of the folder containing required data for tests to run, such as an example forc
    /// project.
    const TEST_DATA_FOLDER_NAME: &str = "test_data";
    /// Name of the library package in the "CARGO_MANIFEST_DIR/TEST_DATA_FOLDER_NAME".
    const TEST_LIBRARY_PACKAGE_NAME: &str = "test_library";
    /// Name of the contract package in the "CARGO_MANIFEST_DIR/TEST_DATA_FOLDER_NAME".
    const TEST_CONTRACT_PACKAGE_NAME: &str = "test_contract";
    /// Name of the predicate package in the "CARGO_MANIFEST_DIR/TEST_DATA_FOLDER_NAME".
    const TEST_PREDICATE_PACKAGE_NAME: &str = "test_predicate";
    /// Name of the script package in the "CARGO_MANIFEST_DIR/TEST_DATA_FOLDER_NAME".
    const TEST_SCRIPT_PACKAGE_NAME: &str = "test_script";

    /// Build the tests in the test package with the given name located at
    /// "CARGO_MANIFEST_DIR/TEST_DATA_FOLDER_NAME/TEST_LIBRARY_PACKAGE_NAME".
    fn test_package_built_tests(package_name: &str) -> anyhow::Result<BuiltTests> {
        let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
        let library_package_dir = PathBuf::from(cargo_manifest_dir)
            .join(TEST_DATA_FOLDER_NAME)
            .join(package_name);
        let library_package_dir_string = library_package_dir.to_string_lossy().to_string();
        let build_options = TestOpts {
            pkg: forc_pkg::PkgOpts {
                path: Some(library_package_dir_string),
                ..Default::default()
            },
            ..Default::default()
        };
        build(build_options)
    }

    fn test_package_test_results(
        package_name: &str,
        test_filter: Option<TestFilter>,
    ) -> anyhow::Result<Vec<TestResult>> {
        let built_tests = test_package_built_tests(package_name)?;
        let test_runner_count = crate::TestRunnerCount::Auto;
        let tested = built_tests.run(test_runner_count, test_filter, GasCostsValues::default())?;
        match tested {
            crate::Tested::Package(tested_pkg) => Ok(tested_pkg.tests),
            crate::Tested::Workspace(_) => {
                unreachable!("test_library is a package, not a workspace.")
            }
        }
    }

    #[test]
    fn test_filter_exact_match() {
        let filter_phrase = "test_bam";
        let test_filter = TestFilter {
            filter_phrase,
            exact_match: true,
        };

        let test_library_results =
            test_package_test_results(TEST_LIBRARY_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_library_test_count = test_library_results.len();

        let test_contract_results =
            test_package_test_results(TEST_CONTRACT_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_contract_test_count = test_contract_results.len();

        let test_predicate_results =
            test_package_test_results(TEST_PREDICATE_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_predicate_test_count = test_predicate_results.len();

        let test_script_results =
            test_package_test_results(TEST_SCRIPT_PACKAGE_NAME, Some(test_filter)).unwrap();
        let tested_script_test_count = test_script_results.len();

        assert_eq!(tested_library_test_count, 1);
        assert_eq!(tested_contract_test_count, 1);
        assert_eq!(tested_predicate_test_count, 1);
        assert_eq!(tested_script_test_count, 1);
    }

    #[test]
    fn test_filter_exact_match_all_ignored() {
        let filter_phrase = "test_ba";
        let test_filter = TestFilter {
            filter_phrase,
            exact_match: true,
        };

        let test_library_results =
            test_package_test_results(TEST_LIBRARY_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_library_test_count = test_library_results.len();

        let test_contract_results =
            test_package_test_results(TEST_CONTRACT_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_contract_test_count = test_contract_results.len();

        let test_predicate_results =
            test_package_test_results(TEST_PREDICATE_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_predicate_test_count = test_predicate_results.len();

        let test_script_results =
            test_package_test_results(TEST_SCRIPT_PACKAGE_NAME, Some(test_filter)).unwrap();
        let tested_script_test_count = test_script_results.len();

        assert_eq!(tested_library_test_count, 0);
        assert_eq!(tested_contract_test_count, 0);
        assert_eq!(tested_predicate_test_count, 0);
        assert_eq!(tested_script_test_count, 0);
    }

    #[test]
    fn test_filter_match_all_ignored() {
        let filter_phrase = "this_test_does_not_exists";
        let test_filter = TestFilter {
            filter_phrase,
            exact_match: false,
        };

        let test_library_results =
            test_package_test_results(TEST_LIBRARY_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_library_test_count = test_library_results.len();

        let test_contract_results =
            test_package_test_results(TEST_CONTRACT_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_contract_test_count = test_contract_results.len();

        let test_predicate_results =
            test_package_test_results(TEST_PREDICATE_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_predicate_test_count = test_predicate_results.len();

        let test_script_results =
            test_package_test_results(TEST_SCRIPT_PACKAGE_NAME, Some(test_filter)).unwrap();
        let tested_script_test_count = test_script_results.len();

        assert_eq!(tested_library_test_count, 0);
        assert_eq!(tested_contract_test_count, 0);
        assert_eq!(tested_predicate_test_count, 0);
        assert_eq!(tested_script_test_count, 0);
    }

    #[test]
    fn test_filter_one_match() {
        let filter_phrase = "test_ba";
        let test_filter = TestFilter {
            filter_phrase,
            exact_match: false,
        };

        let test_library_results =
            test_package_test_results(TEST_LIBRARY_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_library_test_count = test_library_results.len();

        let test_contract_results =
            test_package_test_results(TEST_CONTRACT_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_contract_test_count = test_contract_results.len();

        let test_predicate_results =
            test_package_test_results(TEST_PREDICATE_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_predicate_test_count = test_predicate_results.len();

        let test_script_results =
            test_package_test_results(TEST_SCRIPT_PACKAGE_NAME, Some(test_filter)).unwrap();
        let tested_script_test_count = test_script_results.len();

        assert_eq!(tested_library_test_count, 1);
        assert_eq!(tested_contract_test_count, 1);
        assert_eq!(tested_predicate_test_count, 1);
        assert_eq!(tested_script_test_count, 1);
    }

    #[test]
    fn test_filter_all_match() {
        let filter_phrase = "est_b";
        let test_filter = TestFilter {
            filter_phrase,
            exact_match: false,
        };

        let test_library_results =
            test_package_test_results(TEST_LIBRARY_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_library_test_count = test_library_results.len();

        let test_contract_results =
            test_package_test_results(TEST_CONTRACT_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_contract_test_count = test_contract_results.len();

        let test_predicate_results =
            test_package_test_results(TEST_PREDICATE_PACKAGE_NAME, Some(test_filter.clone()))
                .unwrap();
        let tested_predicate_test_count = test_predicate_results.len();

        let test_script_results =
            test_package_test_results(TEST_SCRIPT_PACKAGE_NAME, Some(test_filter)).unwrap();
        let tested_script_test_count = test_script_results.len();

        assert_eq!(tested_library_test_count, 2);
        assert_eq!(tested_contract_test_count, 2);
        assert_eq!(tested_predicate_test_count, 2);
        assert_eq!(tested_script_test_count, 2);
    }

    #[test]
    fn test_no_filter() {
        let test_filter = None;

        let test_library_results =
            test_package_test_results(TEST_LIBRARY_PACKAGE_NAME, test_filter.clone()).unwrap();
        let tested_library_test_count = test_library_results.len();

        let test_contract_results =
            test_package_test_results(TEST_CONTRACT_PACKAGE_NAME, test_filter.clone()).unwrap();
        let tested_contract_test_count = test_contract_results.len();

        let test_predicate_results =
            test_package_test_results(TEST_PREDICATE_PACKAGE_NAME, test_filter.clone()).unwrap();
        let tested_predicate_test_count = test_predicate_results.len();

        let test_script_results =
            test_package_test_results(TEST_SCRIPT_PACKAGE_NAME, test_filter).unwrap();
        let tested_script_test_count = test_script_results.len();

        assert_eq!(tested_library_test_count, 2);
        assert_eq!(tested_contract_test_count, 2);
        assert_eq!(tested_predicate_test_count, 2);
        assert_eq!(tested_script_test_count, 2);
    }
}
