use libtest_mimic::{Arguments, Trial};
use std::{path::PathBuf, sync::Once};

static FORC_COMPILATION: Once = Once::new();

fn compile_forc() {
    let args = vec!["b", "--release", "-p", "forc"];
    let o = std::process::Command::new("cargo")
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success());
}

pub fn main() {
    let mut args = Arguments::from_args();
    args.nocapture = true;

    let tests = discover_test()
        .into_iter()
        .map(|dir| {
            let manifest_dir = "src/e2e_vm_tests/test_programs/";
            let name = dir.to_str().unwrap().to_string().replace(manifest_dir, "");
            Trial::test(name, move || {
                FORC_COMPILATION.call_once(|| {
                    compile_forc();
                });

                let root = dir.to_str().unwrap();

                let args = vec!["build", "--path", root];
                let o = std::process::Command::new("../target/release/forc")
                    .args(args)
                    .output()
                    .unwrap();

                let snapshot = clean_output(&format!(
                    "exit status: {}\nstdout:\n{}\nstderr:\n{}",
                    o.status.code().unwrap(),
                    String::from_utf8(o.stdout).unwrap(),
                    String::from_utf8(o.stderr).unwrap()
                ));

                fn stdout(root: &str, snapshot: &str) {
                    let mut insta = insta::Settings::new();
                    insta.set_snapshot_path(root);
                    insta.set_prepend_module_to_snapshot(false);
                    insta.set_omit_expression(true);
                    let scope = insta.bind_to_scope();
                    insta::assert_snapshot!("stdout", snapshot);
                    drop(scope);
                }
                stdout(&format!("../{root}"), &snapshot);

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
    raw.replace(&format!("{}/", parent.display()), "")
}
