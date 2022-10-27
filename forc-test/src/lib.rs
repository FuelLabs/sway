use forc_pkg as pkg;
use fuel_tx as tx;
use fuel_vm as vm;
use rand::{Rng, SeedableRng};

/// The result of a `forc test` invocation.
#[derive(Debug)]
pub enum Tested {
    Package(Box<TestedPackage>),
    Workspace,
}

/// The result of testing a specific package.
#[derive(Debug)]
pub struct TestedPackage {
    pub built: pkg::BuiltPackage,
    /// The resulting `ProgramState` after executing the test.
    pub tests: Vec<TestResult>,
}

/// The result of executing a single test within a single package.
// TODO: This should include the function path, span and expected result.
#[derive(Debug)]
pub struct TestResult {
    /// The resulting state after executing the test function.
    pub state: vm::state::ProgramState,
}

/// The set of options provided to the `test` function.
#[derive(Default)]
pub struct Opts {
    pub pkg: pkg::PkgOpts,
    pub print: pkg::PrintOpts,
    pub minify: pkg::MinifyOpts,
    /// If set, outputs a binary file representing the script bytes.
    pub binary_outfile: Option<String>,
    /// If set, outputs source file mapping in JSON format
    pub debug_outfile: Option<String>,
    /// Name of the build profile to use.
    /// If it is not specified, forc will use debug build profile.
    pub build_profile: Option<String>,
    /// Use release build plan. If a custom release plan is not specified, it is implicitly added to the manifest file.
    ///
    ///  If --build-profile is also provided, forc omits this flag and uses provided build-profile.
    pub release: bool,
    /// Output the time elapsed over each part of the compilation process.
    pub time_phases: bool,
}

impl Opts {
    /// Convet this set of test options into a set of build options.
    pub fn into_build_opts(self) -> pkg::BuildOpts {
        pkg::BuildOpts {
            pkg: self.pkg,
            print: self.print,
            minify: self.minify,
            binary_outfile: self.binary_outfile,
            debug_outfile: self.debug_outfile,
            build_profile: self.build_profile,
            release: self.release,
            time_phases: self.time_phases,
            tests: true,
        }
    }
}

/// Build the the given package and run its tests, returning the results.
pub fn test(opts: Opts) -> anyhow::Result<Tested> {
    let build_opts = opts.into_build_opts();

    let built_pkg = match pkg::build_with_options(build_opts)? {
        pkg::Built::Package(pkg) => pkg,
        pkg::Built::Workspace => todo!("run all tests in all workspace members"),
    };

    if !matches!(
        built_pkg.tree_type,
        sway_core::language::parsed::TreeType::Library { .. }
    ) {
        anyhow::bail!("Unstable unit testing only supports tests in libraries for now");
    }

    // TODO: Execute each test function within an interpreter instance.
    // For now we are just executing it as though it were a normal script until we can work out how
    // to iterate over and enter via test function entry points.

    // Create a transaction to execute the test function.
    let script_input_data = vec![];
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x54A9u64);
    let maturity = 1;
    let block_height = (u32::MAX >> 1) as u64;
    let params = tx::ConsensusParameters {
        // The default max length is 1MB which isn't enough for bigger tests.
        max_script_length: 64 * 1024 * 1024,
        ..tx::ConsensusParameters::DEFAULT
    };
    let secret_key = rng.gen();
    let utxo_id = rng.gen();
    let amount = 1;
    let asset_id = Default::default();
    let tx_ptr = rng.gen();
    let tx = tx::TransactionBuilder::script(built_pkg.bytecode.clone(), script_input_data)
        .add_unsigned_coin_input(secret_key, utxo_id, amount, asset_id, tx_ptr, 0)
        .gas_limit(tx::ConsensusParameters::DEFAULT.max_gas_per_tx)
        .maturity(maturity)
        .finalize_checked(block_height as tx::Word, &params);

    // Setup the interpreter.
    let storage = vm::storage::MemoryStorage::default();
    let mut interpreter = vm::interpreter::Interpreter::with_storage(storage, params);
    let transition = interpreter.transact(tx).unwrap();

    // Return the results.
    let state = *transition.state();
    let result = TestResult { state };
    let built = built_pkg;
    let tests = vec![result];
    let tested_pkg = TestedPackage { built, tests };
    let tested = Tested::Package(Box::new(tested_pkg));
    Ok(tested)
}
