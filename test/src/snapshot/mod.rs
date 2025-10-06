use anyhow::Result;
use libtest_mimic::{Arguments, Trial};
use normalize_path::NormalizePath;
use regex::{Captures, Regex};
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Once,
};
use sway_core::Engines;
use sway_features::ExperimentalFeatures;
use sway_ir::{function_print, Backtrace};

static FORC_COMPILATION: Once = Once::new();
static FORC_DOC_COMPILATION: Once = Once::new();

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

                let _ = writeln!(snapshot, "> {cmd}");

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
                    } else if let Some(args) = cmd.strip_prefix("filter-fn ") {
                        if let Some(output) = last_output.take() {
                            let (name, fns) = args.trim().split_once(" ").unwrap();

                            let fns = fns
                                .split(",")
                                .map(|x| x.trim().to_string())
                                .collect::<BTreeSet<String>>();

                            let mut captured = String::new();

                            let mut inside_ir = false;
                            let mut inside_asm = false;
                            let mut last_asm_lines = VecDeque::new();
                            let mut capture_line = false;

                            let compiling_project_line = format!("Compiling script {name}");
                            for line in output.lines() {
                                if line.contains(&compiling_project_line) {
                                    inside_ir = true;
                                }

                                if line.contains(";; ASM: Final program") {
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
                                                if fns.contains(f.get_name(&ir)) {
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
                                    if line.contains("save locals base register for function") {
                                        for f in fns.iter() {
                                            if line.contains(f.as_str()) {
                                                capture_line = true;

                                                snapshot.push('\n');

                                                for l in last_asm_lines.drain(..) {
                                                    snapshot.push_str(l);
                                                    snapshot.push('\n');
                                                }
                                            }
                                        }
                                    }

                                    // keep the last two lines
                                    if last_asm_lines.len() >= 2 {
                                        last_asm_lines.pop_front();
                                    }
                                    last_asm_lines.push_back(line);

                                    if line.is_empty() {
                                        inside_asm = false;
                                    }

                                    if line.contains("; return from call") {
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
                    },
                }
            }
            _ => {
                panic!("`cmds` items can only be strings or inline tables.");
            },
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
