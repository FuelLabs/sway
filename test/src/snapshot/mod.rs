use anyhow::Result;
use fuel_vm::{
    prelude::{Finalizable, GasCostsValues, TransactionBuilderExt as _},
    state::ProgramState,
    storage::MemoryStorage,
};
use fuels::{
    crypto::SecretKey,
    tx::{ConsensusParameters, GasCosts, Receipt, ScriptParameters, TxParameters},
    types::gas_price::LatestGasPrice,
};
use gimli::{DebugLine, LittleEndian, Reader};
use libtest_mimic::{Arguments, Trial};
use normalize_path::NormalizePath;
use object::{Object, ObjectSection};
use rand::{Rng as _, SeedableRng as _};
use regex::{Captures, Regex};
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    error::Error,
    io::{Seek, Write},
    ops::ControlFlow,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Once,
    u64,
};
use sway_core::{asm_generation::fuel, Engines};
use sway_features::ExperimentalFeatures;
use sway_ir::{function_print, Backtrace};

static FORC_COMPILATION: Once = Once::new();
static FORC_DOC_COMPILATION: Once = Once::new();
static FORC_MIGRATE_COMPILATION: Once = Once::new();

fn compile_forc() {
    let args = vec!["b", "--release", "-p", "forc"];
    let o = std::process::Command::new("cargo")
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success());
}

fn compile_forc_doc() {
    let args = vec!["b", "--release", "-p", "forc-doc"];
    let o = std::process::Command::new("cargo")
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success());
}

fn compile_forc_migrate() {
    let args = vec!["b", "--release", "-p", "forc-migrate"];
    let o = std::process::Command::new("cargo")
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success());
}

#[derive(Default)]
struct UndoFiles {
    contents: HashMap<PathBuf, Vec<u8>>,
}

impl Drop for UndoFiles {
    fn drop(&mut self) {
        #[allow(clippy::iter_over_hash_type)]
        for (path, contents) in self.contents.drain() {
            let _ = std::fs::write(path, contents);
        }
    }
}

pub(super) async fn run(filter_regex: Option<&regex::Regex>) -> Result<()> {
    let repo_root = PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR")?)?
        .parent()
        .unwrap()
        .to_path_buf();
    let test_root = repo_root.join("test");
    let test_programs_dir = test_root.join("src/e2e_vm_tests/test_programs/");

    let args = Arguments {
        filter: filter_regex.as_ref().map(|filter| filter.to_string()),
        nocapture: true,
        ..Default::default()
    };

    let tests = discover_tests(&test_root)
        .into_iter()
        .map(|dir| {
            let name = dir
                .strip_prefix(&test_programs_dir)
                .unwrap()
                .display()
                .to_string();

            let repo_root = repo_root.clone();
            Trial::test(name.clone(), move || {
                let snapshot_toml =
                    std::fs::read_to_string(format!("{}/snapshot.toml", dir.display()))?;
                let snapshot_toml = if snapshot_toml.trim().is_empty() {
                    "cmds = [ \"forc build --path {root}\" ]".to_string()
                } else {
                    snapshot_toml
                };

                let snapshot_toml = toml::from_str::<toml::Value>(&snapshot_toml)?;
                let root = dir.strip_prefix(&repo_root).unwrap().display().to_string();

                let cmds = snapshot_toml.get("cmds").unwrap().as_array().unwrap();

                let mut snapshot = String::new();

                let _ = run_cmds(&name, &repo_root, &root, cmds, &mut snapshot);

                fn stdout(root: &str, snapshot: &str) {
                    let root = PathBuf::from_str(root).unwrap();
                    let root = root.normalize();

                    let mut insta = insta::Settings::new();
                    insta.set_snapshot_path(root);
                    insta.set_prepend_module_to_snapshot(false);
                    insta.set_omit_expression(true);
                    let scope = insta.bind_to_scope();
                    insta::assert_snapshot!("stdout", snapshot);
                    drop(scope);
                }
                stdout(&format!("{}/{root}", repo_root.display()), &snapshot);

                Ok(())
            })
        })
        .collect();

    libtest_mimic::run(&args, tests).exit();
}

fn run_cmds(
    test_name: &String,
    repo_root: &PathBuf,
    root: &String,
    cmds: &Vec<toml::Value>,
    snapshot: &mut String,
) -> std::result::Result<(), libtest_mimic::Failed> {
    use std::fmt::Write;

    let name = PathBuf::from_str(test_name).unwrap();
    let name = name.file_stem().unwrap();

    let find_blocks_regex = Regex::new(r#"START ([0-9a-zA-Z_]*)"#).unwrap();

    for cmd in cmds {
        match cmd {
            toml::Value::String(cmd) => {
                let cmd = cmd
                    .replace("{root}", root)
                    .replace("{name}", name.to_str().unwrap());

                if !cmd.starts_with("echo") {
                    let _ = writeln!(snapshot, "> {cmd}");
                }

                let mut last_output: Option<String> = None;

                // We intentionally split the command by " | " to allow for
                // `regex` command to support `|` operator, although without
                // surrounding spaces.
                for cmd in cmd.split(" | ") {
                    let cmd = cmd.trim();

                    let cmd = if let Some(cmd) = cmd.strip_prefix("forc doc ") {
                        FORC_DOC_COMPILATION.call_once(|| {
                            compile_forc_doc();
                        });
                        format!("target/release/forc-doc {cmd} 1>&2")
                    } else if let Some(cmd) = cmd.strip_prefix("forc migrate ") {
                        FORC_MIGRATE_COMPILATION.call_once(|| {
                            compile_forc_migrate();
                        });
                        format!("target/release/forc-migrate {cmd} 1>&2")
                    } else if let Some(cmd) = cmd.strip_prefix("forc ") {
                        FORC_COMPILATION.call_once(|| {
                            compile_forc();
                        });
                        format!("target/release/forc {cmd} 1>&2")
                    } else if let Some(cmd) = cmd.strip_prefix("sub ") {
                        let arg = cmd.trim();
                        if let Some(l) = last_output.take() {
                            let mut new_output = String::new();
                            for line in l.lines() {
                                if line.contains(arg) {
                                    new_output.push_str(line);
                                    new_output.push('\n');
                                }
                            }
                            last_output = Some(new_output);
                        }
                        continue;
                    } else if let Some(cmd) = cmd.strip_prefix("regex ") {
                        let arg = cmd.trim();
                        let arg = arg.trim_matches('\'');
                        let regex = Regex::new(arg).expect("regex provided to the snapshot `regex` filter is not a valid Rust regex");
                        if let Some(l) = last_output.take() {
                            let mut new_output = String::new();
                            for line in l.lines() {
                                if regex.is_match(line) {
                                    new_output.push_str(line);
                                    new_output.push('\n');
                                }
                            }
                            last_output = Some(new_output);
                        }
                        continue;
                    } else if let Some(args) = cmd.strip_prefix("replace-file ") {
                        let Some((path, args)) = args.trim().split_once(" ") else {
                            panic!("replace needs three arguments: file from to");
                        };
                        let Some(from) = args.trim().strip_prefix("\"") else {
                            panic!("replace arguments must be quoted");
                        };
                        let Some((from, args)) = from.split_once("\"") else {
                            panic!("replace arguments must be quoted");
                        };

                        let Some(to) = args.trim().strip_prefix("\"") else {
                            panic!("replace arguments must be quoted");
                        };
                        let Some((to, _)) = to.split_once("\"") else {
                            panic!("replace arguments must be quoted");
                        };

                        let proj_root = repo_root.join(root);
                        let path = proj_root.join(path);
                        let path = path.canonicalize().unwrap();

                        if !path
                            .display()
                            .to_string()
                            .starts_with(&proj_root.display().to_string())
                        {
                            panic!("not allowed to edit files outside project folder");
                        }

                        let contents = std::fs::read_to_string(&path).unwrap();
                        let contents = contents.replace(from, to);
                        std::fs::write(path, contents).unwrap();

                        continue;
                    } else if let Some(args) = cmd.strip_prefix("filter-fn ") {
                        if let Some(output) = last_output.take() {
                            if !output.starts_with("exit status: 0") {
                                last_output = Some(output);
                                continue;
                            }

                            let (name, fns) = args.trim().split_once(" ").unwrap();

                            let fns = fns
                                .split(",")
                                .map(|x| x.trim().to_string())
                                .collect::<BTreeSet<String>>();

                            let mut captured = String::new();

                            let mut inside_ir = false;
                            let mut inside_asm = false;
                            let mut capture_line = false;

                            let compiling_project_line = format!("Compiling script {name}");
                            for line in output.lines() {
                                if line.contains(&compiling_project_line) {
                                    inside_ir = true;
                                }

                                if line.contains(";; ASM:") {
                                    inside_asm = true;
                                }

                                if inside_ir {
                                    if line.starts_with("// IR:") {
                                        capture_line = true;
                                    }

                                    if line.starts_with("!0 =") {
                                        let engines = Engines::default();
                                        let ir = sway_ir::parse(
                                            &captured,
                                            engines.se(),
                                            ExperimentalFeatures::default(),
                                            Backtrace::None,
                                        )
                                        .unwrap();

                                        for m in ir.module_iter() {
                                            for f in m.function_iter(&ir) {
                                                let fn_name = f.get_name(&ir);
                                                let any = fns.iter().any(|x| fn_name.contains(x));
                                                if any {
                                                    snapshot.push('\n');
                                                    function_print(snapshot, &ir, f, false)
                                                        .unwrap();
                                                    snapshot.push('\n');
                                                }
                                            }
                                        }

                                        capture_line = false;
                                        inside_ir = false;
                                        captured.clear();
                                    }
                                }

                                if inside_asm {
                                    if (line.contains("fn init:") || line.contains("entry init"))
                                        && fns.iter().any(|f| line.contains(&format!("init: {f}")))
                                    {
                                        capture_line = true;

                                        snapshot.push('\n');
                                    }

                                    if line.is_empty() {
                                        inside_asm = false;
                                    }

                                    if line.contains("end:") && line.contains("] return") {
                                        if capture_line {
                                            captured.push_str(line);
                                            captured.push('\n');

                                            write!(snapshot, "{captured}").unwrap();
                                            captured.clear();
                                        }

                                        capture_line = false;
                                    }
                                }

                                if capture_line {
                                    captured.push_str(line);
                                    captured.push('\n');
                                }
                            }

                            last_output = Some(String::new());
                        }
                        continue;
                    } else if let Some(args) = cmd.strip_prefix("echo ") {
                        let mut chars = args.chars();
                        'nextline: loop {
                            for _ in 0..80 {
                                if let Some(c) = chars.next() {
                                    snapshot.push(c);
                                } else {
                                    break 'nextline;
                                }
                            }

                            for c in chars.by_ref() {
                                if c == ' ' || c == '\n' {
                                    snapshot.push('\n');
                                    continue 'nextline;
                                } else {
                                    snapshot.push(c);
                                }
                            }

                            break 'nextline;
                        }

                        snapshot.push('\n');

                        continue;
                    } else if let Some(args) = cmd.strip_prefix("patch-bin ") {
                        if let Err(err) = patch_bin_command(repo_root, root, args) {
                            snapshot.push_str(&format!("{err:#?}"));
                        }
                        continue;
                    } else if let Some(args) = cmd.strip_prefix("fuel-vm ") {
                        if let Err(err) = fuel_vm_command(snapshot, args) {
                            snapshot.push_str(&format!("{err:#?}"));
                        }
                        continue;
                    } else {
                        panic!("`{cmd}` is not a supported snapshot command.\nPossible tool commands: forc doc, forc\nPossible filtering commands: sub, regex, filter-fn");
                    };

                    let o = duct::cmd!("bash", "-c", cmd.clone())
                        .dir(repo_root.clone())
                        .stderr_to_stdout()
                        .stdout_capture();

                    let o = if let Some(last_output) = last_output.as_ref() {
                        o.stdin_bytes(last_output.as_bytes())
                    } else {
                        o
                    };

                    let o = o.env("COLUMNS", "10").unchecked().start().unwrap();
                    let o = o.wait().unwrap();
                    last_output = Some(clean_output(&format!(
                        "exit status: {}\noutput:\n{}",
                        o.status.code().unwrap(),
                        std::str::from_utf8(&o.stdout).unwrap(),
                    )));
                }

                let _ = writeln!(snapshot, "{}", last_output.unwrap_or_default());
            }
            toml::Value::Table(map) => {
                let repeat_type = map["repeat"].as_str().unwrap();
                let cmds = map["cmds"].as_array().unwrap();

                match repeat_type {
                    "for-each-block" => {
                        fn remove_block_from_file(contents: &str, block_name: &str) -> String {
                            let block_regex = Regex::new(&format!("\\/\\* START {block_name} \\*\\/[.\\s\\S]+?END {block_name} \\*\\/")).unwrap();
                            block_regex
                                .replace_all(contents, |_: &Captures| -> String { String::new() })
                                .to_string()
                        }

                        let path = PathBuf::from_str(root).unwrap().join("src/main.sw");
                        let byte_contents = std::fs::read(&path).unwrap();
                        let contents = String::from_utf8(byte_contents.clone()).unwrap();

                        let mut blocks = BTreeSet::new();

                        for capture in find_blocks_regex.captures_iter(&contents) {
                            let name = capture.get(1).unwrap().as_str().to_string();
                            blocks.insert(name);
                        }

                        for block in blocks.iter() {
                            let _ = writeln!(snapshot, "> Block: {block}");

                            let mut undo = UndoFiles::default();
                            undo.contents.insert(path.clone(), byte_contents.clone());

                            let mut new_contents = contents.clone();
                            for remove_block in blocks.iter() {
                                if remove_block != block {
                                    new_contents =
                                        remove_block_from_file(&new_contents, remove_block)
                                            .to_string();
                                }
                            }

                            let _ = std::fs::write(&path, new_contents);
                            let _ = run_cmds(test_name, repo_root, root, cmds, snapshot);
                        }
                    }
                    _ => {
                        panic!("`{cmd}` is not a supported repeat type.\nPossible types are: for-each-block.");
                    }
                }
            }
            _ => {
                panic!("`cmds` items can only be strings or inline tables.");
            }
        }
    }

    Ok(())
}

fn fuel_vm_command(snapshot: &mut String, args: &str) -> Result<(), std::io::Error> {
    let mut args = args.split(" ");
    match args.next() {
        Some("run") => {
            let bin_file = args.next().unwrap();

            let bytecode = std::fs::read(bin_file)?;
            let script_input_data = vec![];

            const TEST_METADATA_SEED: u64 = 0x7E57u64;
            let rng = &mut rand::rngs::StdRng::seed_from_u64(TEST_METADATA_SEED);

            // Prepare the transaction metadata.
            let secret_key = SecretKey::random(rng);
            let utxo_id = rng.r#gen();
            let amount = 1;
            let maturity = 1.into();

            let asset_id = fuels::types::AssetId::BASE;
            let tx_pointer = rng.r#gen();

            let gas_costs = forc_test::GasCostsSource::BuiltIn
                .provide_gas_costs()
                .unwrap();
            let consensus_params = maxed_consensus_params(gas_costs.clone());
            let mut tx_builder =
                fuel_vm::prelude::TransactionBuilder::script(bytecode, script_input_data);
            tx_builder
                .with_gas_costs(GasCosts::new(gas_costs))
                .script_gas_limit(u64::MAX)
                .with_params(consensus_params.clone())
                .add_unsigned_coin_input(secret_key, utxo_id, amount, asset_id, tx_pointer)
                .maturity(maturity);
            let block_height = (u32::MAX >> 1).into();
            let tx = tx_builder
                .finalize_checked(block_height)
                .into_ready(
                    0,
                    consensus_params.gas_costs(),
                    consensus_params.fee_params(),
                    None,
                )
                .unwrap();

            let interpreter_params =
                fuel_vm::interpreter::InterpreterParams::new(0, consensus_params);
            let memory_instance = fuel_vm::prelude::MemoryInstance::new();
            let storage = MemoryStorage::default();
            let mut interpreter: fuel_vm::prelude::Interpreter<
                fuel_vm::prelude::MemoryInstance,
                MemoryStorage,
                fuel_vm::prelude::Script,
                forc_test::ecal::EcalSyscallHandler,
            > = fuel_vm::prelude::Interpreter::with_storage(
                memory_instance,
                storage,
                interpreter_params,
            );

            interpreter.ecal_state_mut().clear();

            // Run test until its end
            interpreter.set_single_stepping(true);
            let mut state = {
                let transition = interpreter.transact(tx.clone());
                Ok(*transition.unwrap().state())
            };

            loop {
                interpreter.set_single_stepping(true);
                match state {
                    Err(_) => {
                        state = Ok(ProgramState::Revert(0));
                        break;
                    }
                    Ok(
                        ProgramState::Return(_)
                        | ProgramState::ReturnData(_)
                        | ProgramState::Revert(_),
                    ) => break,
                    Ok(ProgramState::RunProgram(_) | ProgramState::VerifyPredicate(_)) => {
                        state = interpreter.resume();
                    }
                }
            }

            let (gas_used, logs) = get_gas_and_receipts(interpreter.receipts().to_vec()).unwrap();
            snapshot.push_str(&format!("Gas: {}\nLogs:\n", gas_used));
            for l in logs {
                snapshot.push_str(&format!("{l:#?}\n"));
            }
        }
        _ => panic!("unknown command"),
    }

    Ok(())
}

pub(crate) fn maxed_consensus_params(gas_costs_values: GasCostsValues) -> ConsensusParameters {
    let script_params = ScriptParameters::DEFAULT
        .with_max_script_length(u64::MAX)
        .with_max_script_data_length(u64::MAX);
    let tx_params = TxParameters::DEFAULT
        .with_max_gas_per_tx(u64::MAX)
        .with_max_size(u64::MAX);
    let contract_params = fuels::tx::ContractParameters::DEFAULT
        .with_contract_max_size(u64::MAX)
        .with_max_storage_slots(u64::MAX);
    ConsensusParameters::V1(fuels::tx::consensus_parameters::ConsensusParametersV1 {
        script_params,
        tx_params,
        contract_params,
        gas_costs: gas_costs_values.into(),
        block_gas_limit: u64::MAX,
        ..Default::default()
    })
}

fn get_gas_and_receipts(receipts: Vec<Receipt>) -> anyhow::Result<(u64, Vec<Receipt>)> {
    let gas_used = *receipts
        .iter()
        .find_map(|receipt| match receipt {
            fuels::tx::Receipt::ScriptResult { gas_used, .. } => Some(gas_used),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("missing used gas information from test execution"))?;

    // Only retain `Log` and `LogData` receipts.
    let logs = receipts
        .into_iter()
        // .filter(|receipt| {
        //     matches!(receipt, fuels::tx::Receipt::Log { .. })
        //         || matches!(receipt, fuels::tx::Receipt::LogData { .. })
        // })
        .collect();
    Ok((gas_used, logs))
}

fn patch_bin_command(repo_root: &PathBuf, root: &String, args: &str) -> Result<(), std::io::Error> {
    let proj_root = repo_root.join(root);
    let build = args.trim();
    let out_dir = proj_root.join("out").join(build);

    let bin_file = std::fs::read_dir(&out_dir)?
        .into_iter()
        .flatten()
        .find(|x| x.path().extension().and_then(|x| x.to_str()) == Some("bin"));
    let dwarf_file = std::fs::read_dir(&out_dir)?
        .into_iter()
        .flatten()
        .find(|x| x.path().extension().and_then(|x| x.to_str()) == Some("obj"))
        .unwrap();

    let file = std::fs::read(dwarf_file.path()).unwrap();
    let file = object::File::parse(&*file).unwrap();

    dump_file(
        &file,
        if file.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        },
        &bin_file.unwrap().path(),
    )
    .unwrap();

    Ok(())
}

fn dump_file(
    object: &object::File,
    endian: gimli::RunTimeEndian,
    bin_file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let bin_file = std::fs::read(bin_file_path).unwrap();

    // Load a section and return as `Cow<[u8]>`.
    let load_section = |id: gimli::SectionId| -> Result<Cow<[u8]>, Box<dyn std::error::Error>> {
        Ok(match object.section_by_name(id.name()) {
            Some(section) => section.uncompressed_data()?,
            None => Cow::Borrowed(&[]),
        })
    };

    // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
    let borrow_section = |section| gimli::EndianSlice::new(Cow::as_ref(section), endian);

    // Load all of the sections.
    let dwarf_sections = gimli::DwarfSections::load(&load_section)?;

    // Create `EndianSlice`s for all of the sections.
    let dwarf = dwarf_sections.borrow(borrow_section);

    // Iterate over the compilation units.
    let mut iter = dwarf.units();
    while let Some(header) = iter.next()? {
        println!(
            "Line number info for unit at <.debug_info+0x{:?}>",
            header.offset()
        );
        let unit = dwarf.unit(header)?;
        let unit = unit.unit_ref(&dwarf);

        // Get the line program for the compilation unit.
        if let Some(program) = unit.line_program.clone() {
            let comp_dir = if let Some(ref dir) = unit.comp_dir {
                PathBuf::from(dir.to_string_lossy().into_owned())
            } else {
                PathBuf::new()
            };

            // Iterate over the line program rows.
            let mut rows = program.rows();
            while let Some((header, row)) = rows.next_row()? {
                if row.end_sequence() {
                    // End of sequence indicates a possible gap in addresses.
                    println!("{:x} end-sequence", row.address());
                } else {
                    // Determine the path. Real applications should cache this for performance.
                    let mut path = PathBuf::new();
                    if let Some(file) = row.file(header) {
                        path.clone_from(&comp_dir);

                        // The directory index 0 is defined to correspond to the compilation unit directory.
                        if file.directory_index() != 0 {
                            if let Some(dir) = file.directory(header) {
                                path.push(unit.attr_string(dir)?.to_string_lossy().as_ref());
                            }
                        }

                        path.push(
                            unit.attr_string(file.path_name())?
                                .to_string_lossy()
                                .as_ref(),
                        );
                    }

                    // Determine line/column. DWARF line/column is never 0, so we use that
                    // but other applications may want to display this differently.
                    let line = match row.line() {
                        Some(line) => line.get(),
                        None => 0,
                    };
                    let column = match row.column() {
                        gimli::ColumnType::LeftEdge => 0,
                        gimli::ColumnType::Column(column) => column.get(),
                    };

                    println!("{:x} {}:{}:{}", row.address(), path.display(), line, column);

                    let opcode = [
                        bin_file[row.address() as usize * 4 + 0],
                        bin_file[row.address() as usize * 4 + 1],
                        bin_file[row.address() as usize * 4 + 2],
                        bin_file[row.address() as usize * 4 + 3],
                    ];
                    print!("    ");
                    if let Ok(i) = fuel_vm::fuel_asm::Instruction::try_from(opcode) {
                        print!(" {i:?}");
                    }
                    let opcode = u32::from_le_bytes(opcode);
                    println!("{:x}", opcode);

                    let code = std::fs::read_to_string(&path).unwrap();
                    let line = code.lines().skip((line - 1) as usize).next().unwrap();
                    print!("    {line}");

                    if let Some((_, rest)) = line.split_once("// PATCH: ") {
                        let mut args = String::new();
                        let mut i = 0u32;
                        for part in rest.trim().split(" ") {
                            if part.len() == 6 {
                                args.push_str(part);
                            }
                            if let Some(n) = part.strip_prefix("0x") {
                                i = (u8::from_str_radix(n, 16).unwrap() as u32) << 24;
                            }
                        }

                        for (idx, c) in args.chars().enumerate() {
                            let v = if c == '0' { 0 } else { 1 };
                            i |= v << (24 - idx - 1);
                        }

                        if let Ok(i) = fuel_vm::fuel_asm::Instruction::try_from(i) {
                            print!(" will patch to {i:?}");
                        }

                        let mut f = std::fs::File::options()
                            .write(true)
                            .open(bin_file_path)
                            .unwrap();
                        f.seek(std::io::SeekFrom::Start(row.address() as u64 * 4))
                            .unwrap();
                        f.write_all(&i.to_be_bytes()).unwrap();
                        f.flush().unwrap();
                    }

                    println!();
                }
            }
        }
    }
    Ok(())
}

pub fn discover_tests(test_root: &Path) -> Vec<PathBuf> {
    use glob::glob;

    let mut entries = vec![];

    let pattern = format!("{}/**/snapshot.toml", test_root.display());
    for entry in glob(&pattern)
        .expect("Failed to read glob pattern")
        .flatten()
    {
        entries.push(entry.parent().unwrap().to_owned())
    }

    entries
}

fn clean_output(output: &str) -> String {
    #[derive(Default)]
    struct RawText(String);

    impl vte::Perform for RawText {
        fn print(&mut self, c: char) {
            self.0.push(c);
        }

        fn execute(&mut self, _: u8) {}

        fn hook(&mut self, _: &vte::Params, _: &[u8], _: bool, _: char) {}

        fn put(&mut self, b: u8) {
            self.0.push(b as char);
        }

        fn unhook(&mut self) {}

        fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}

        fn csi_dispatch(&mut self, _: &vte::Params, _: &[u8], _: bool, _: char) {}

        fn esc_dispatch(&mut self, _: &[u8], _: bool, _: u8) {}
    }

    let mut raw = String::new();
    for line in output.lines() {
        let mut performer = RawText::default();
        let mut p = vte::Parser::new();
        for b in line.as_bytes() {
            p.advance(&mut performer, *b);
        }
        raw.push_str(&performer.0);
        raw.push('\n');
    }

    // Remove absolute paths from snapshot tests
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_dir: PathBuf = PathBuf::from(manifest_dir);
    let parent = manifest_dir.parent().unwrap();
    let result = raw.replace(&format!("{}/", parent.display()), "");

    // Remove compilation time
    let r = Regex::new("(Finished (debug|release) \\[.*?\\] target\\(s\\) \\[.*?\\] in )(.*?s)")
        .unwrap();
    let result = r.replace_all(&result, "$1???");

    // Remove forc test time
    let r = Regex::new("((F|f)inished in )(.*?s)").unwrap();
    let result = r.replace_all(&result, "$1???");

    // Remove individual test duration time
    let r = Regex::new("(test .+ \\()(.*?s)(, .+ gas\\))").unwrap();
    let result = r.replace_all(&result, "$1???$3");

    // Remove test result "finished in" time
    let r = Regex::new("(test result: .+ finished in )(.*?s)").unwrap();
    let result = r.replace(&result, "$1???");

    // Remove test duration time
    let r = Regex::new("(Finished in )(.*?s)").unwrap();
    let result = r.replace(&result, "$1???");

    result.to_string()
}
