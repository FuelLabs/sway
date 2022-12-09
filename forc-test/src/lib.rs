use std::{collections::HashSet, fs, path::PathBuf, sync::Arc};

use forc_pkg as pkg;
use fuel_tx as tx;
use fuel_vm::{self as vm, prelude::Opcode};
use pkg::BuiltPackage;
use rand::{Rng, SeedableRng};
use sway_core::{language::ty::TyFunctionDeclaration, transform::AttributeKind};
use sway_types::{Span, Spanned};

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
    pub condition: TestPassCondition,
}

/// The possible conditions for a test result to be considered "passing".
#[derive(Debug)]
pub enum TestPassCondition {
    ShouldRevert,
    ShouldNotRevert,
}

/// A package or a workspace that has been built, ready for test execution.
pub enum BuiltTests {
    Package(Box<pkg::BuiltPackage>),
    Workspace(Vec<pkg::BuiltPackage>),
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
    /// If --build-profile is also provided, forc omits this flag and uses provided build-profile.
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
        let pkgs: Vec<&BuiltPackage> = match self {
            BuiltTests::Package(pkg) => vec![pkg],
            BuiltTests::Workspace(workspace) => workspace.iter().collect(),
        };
        pkgs.iter()
            .map(|pkg| pkg.entries.iter().filter(|e| e.is_test()).count())
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
    let built_tests = match pkg::build_with_options(build_opts)? {
        pkg::Built::Package(pkg) => BuiltTests::Package(pkg),
        pkg::Built::Workspace(workspace) => {
            BuiltTests::Workspace(workspace.values().cloned().collect())
        }
    };
    Ok(built_tests)
}

fn test_pass_condition(
    test_function_decl: &TyFunctionDeclaration,
) -> anyhow::Result<TestPassCondition> {
    let test_args: HashSet<String> = test_function_decl
        .attributes
        .get(&AttributeKind::Test)
        .expect("test declaration is missing test attribute")
        .iter()
        .flat_map(|attr| attr.args.iter().map(|arg| arg.to_string()))
        .collect();
    let test_name = &test_function_decl.name;
    if test_args.is_empty() {
        Ok(TestPassCondition::ShouldNotRevert)
    } else if test_args.get("should_revert").is_some() {
        Ok(TestPassCondition::ShouldRevert)
    } else {
        anyhow::bail!("Invalid test argument(s) for test: {test_name}.")
    }
}

/// Build the the given package and run its tests, returning the results.
fn run_tests(built: BuiltTests) -> anyhow::Result<Tested> {
    match built {
        BuiltTests::Package(pkg) => {
            let tested_pkg = run_pkg_tests(&pkg)?;
            Ok(Tested::Package(Box::new(tested_pkg)))
        }
        BuiltTests::Workspace(workspace) => {
            let tested_pkgs = workspace
                .iter()
                .map(run_pkg_tests)
                .collect::<anyhow::Result<Vec<TestedPackage>>>()?;
            Ok(Tested::Workspace(tested_pkgs))
        }
    }
}

fn run_pkg_tests(built_pkg: &BuiltPackage) -> anyhow::Result<TestedPackage> {
    // Run all tests and collect their results.
    // TODO: We can easily parallelise this, but let's wait until testing is stable first.
    let tests = built_pkg
        .entries
        .iter()
        .filter(|entry| entry.is_test())
        .map(|entry| {
            let offset = u32::try_from(entry.imm).expect("test instruction offset out of range");
            let name = entry.fn_name.clone();
            let (state, duration) = exec_test(&built_pkg.bytecode, offset);
            let test_decl_id = entry
                .test_decl_id
                .clone()
                .expect("test entry point is missing declaration id");
            let span = test_decl_id.span();
            let test_function_decl =
                sway_core::declaration_engine::de_get_function(test_decl_id, &span)
                    .expect("declaration engine is missing function declaration for test");
            let condition = test_pass_condition(&test_function_decl)?;
            Ok(TestResult {
                name,
                duration,
                span,
                state,
                condition,
            })
        })
        .collect::<anyhow::Result<_>>()?;

    let tested_pkg = TestedPackage {
        built: Box::new(built_pkg.clone()),
        tests,
    };

    Ok(tested_pkg)
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
    const PROGRAM_START_BYTE_OFFSET: usize = PROGRAM_START_INST_OFFSET as usize * Opcode::LEN;

    // If our desired entry point is the program start, no need to jump.
    if test_offset == PROGRAM_START_INST_OFFSET {
        return std::borrow::Cow::Borrowed(bytecode);
    }

    // Create the jump instruction and splice it into the bytecode.
    let ji = Opcode::JI(test_offset);
    let ji_bytes = ji.to_bytes();
    let start = PROGRAM_START_BYTE_OFFSET;
    let end = start + ji_bytes.len();
    let mut patched = bytecode.to_vec();
    patched.splice(start..end, ji_bytes);
    std::borrow::Cow::Owned(patched)
}

// Execute the test whose entry point is at the given instruction offset as if it were a script.
fn exec_test(bytecode: &[u8], test_offset: u32) -> (vm::state::ProgramState, std::time::Duration) {
    // Patch the bytecode to jump to the relevant test.
    let bytecode = patch_test_bytecode(bytecode, test_offset).into_owned();

    // Create a transaction to execute the test function.
    let script_input_data = vec![];
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x7E57u64);
    let maturity = 1;
    let block_height = (u32::MAX >> 1) as u64;
    let secret_key = rng.gen();
    let utxo_id = rng.gen();
    let amount = 1;
    let asset_id = Default::default();
    let tx_ptr = rng.gen();
    let params = tx::ConsensusParameters::default();
    let tx = tx::TransactionBuilder::script(bytecode, script_input_data)
        .add_unsigned_coin_input(secret_key, utxo_id, amount, asset_id, tx_ptr, 0)
        .gas_limit(tx::ConsensusParameters::DEFAULT.max_gas_per_tx)
        .maturity(maturity)
        .finalize_checked(block_height as tx::Word, &params);

    // Setup the interpreter.
    let storage = vm::storage::MemoryStorage::default();
    let mut interpreter = vm::interpreter::Interpreter::with_storage(storage, params);

    // Execute and return the result.
    let start = std::time::Instant::now();
    let transition = interpreter.transact(tx).unwrap();
    let duration = start.elapsed();
    let state = *transition.state();
    (state, duration)
}
