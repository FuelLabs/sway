use libtest_mimic::{Arguments, Trial};
use normalize_path::NormalizePath;
use regex::Regex;
use std::{path::PathBuf, str::FromStr, sync::Once};

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

pub fn main() {
    let repo_root: PathBuf =
        PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    let repo_root = repo_root.parent().unwrap().to_path_buf();

    let mut args = Arguments::from_args();
    args.nocapture = true;

    let tests = discover_test()
        .into_iter()
        .map(|dir| {
            let test_programs_dir = "src/e2e_vm_tests/test_programs/";
            let name = dir
                .to_str()
                .unwrap()
                .to_string()
                .replace(test_programs_dir, "");

            let repo_root = repo_root.clone();
            Trial::test(name, move || {
                let snapshot_toml =
                    std::fs::read_to_string(format!("{}/snapshot.toml", dir.display()))?;
                let snapshot_toml = toml::from_str::<toml::Value>(&snapshot_toml)?;
                let cmds = if let Some(cmds) = snapshot_toml.get("cmds") {
                    cmds.as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect::<Vec<_>>()
                } else {
                    vec!["forc build --path {root}"]
                };

                let root = format!("test/{}", dir.display());

                use std::fmt::Write;
                let mut snapshot = String::new();

                for cmd in cmds {
                    let cmd = cmd.replace("{root}", &root);

                    let _ = writeln!(&mut snapshot, "> {}", cmd);

                    // known commands
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
                    } else {
                        panic!("Not supported. Possible commands: forc")
                    };

                    let o = duct::cmd!("bash", "-c", cmd.clone())
                        .dir(repo_root.clone())
                        .stderr_to_stdout()
                        .stdout_capture()
                        .env("COLUMNS", "10")
                        .unchecked()
                        .start()
                        .unwrap();
                    let o = o.wait().unwrap();

                    let _ = writeln!(
                        &mut snapshot,
                        "{}",
                        clean_output(&format!(
                            "exit status: {}\noutput:\n{}",
                            o.status.code().unwrap(),
                            std::str::from_utf8(&o.stdout).unwrap(),
                        ))
                    );
                }

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

pub fn discover_test() -> Vec<PathBuf> {
    use glob::glob;

    let mut entries = vec![];

    for entry in glob("**/snapshot.toml")
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
    let r = Regex::new("(Finished release \\[.*?\\] target\\(s\\) \\[.*?\\] in )(.*?s)").unwrap();
    let result = r.replace(&result, "$1???");

    result.to_string()
}
