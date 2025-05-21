use assert_json_diff::assert_json_include;
use futures::StreamExt;
use lsp_types::Url;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use serde_json::Value;
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
};
use tokio::task::JoinHandle;
use tower_lsp::ClientSocket;

pub fn load_sway_example(src_path: PathBuf) -> (Url, String) {
    let mut file = fs::File::open(&src_path).unwrap();
    let mut sway_program = String::new();
    file.read_to_string(&mut sway_program).unwrap();

    let uri = Url::from_file_path(src_path).unwrap();
    (uri, sway_program)
}

pub fn sway_workspace_dir() -> PathBuf {
    env::current_dir().unwrap().parent().unwrap().to_path_buf()
}

pub fn in_language_test_dir() -> PathBuf {
    PathBuf::from("test/src/in_language_tests")
}

pub fn sdk_harness_test_projects_dir() -> PathBuf {
    PathBuf::from("test/src/sdk-harness")
}

pub fn e2e_language_dir() -> PathBuf {
    PathBuf::from("test/src/e2e_vm_tests/test_programs/should_pass/language")
}

pub fn e2e_should_pass_dir() -> PathBuf {
    PathBuf::from("test/src/e2e_vm_tests/test_programs/should_pass")
}

pub fn e2e_should_fail_dir() -> PathBuf {
    PathBuf::from("test/src/e2e_vm_tests/test_programs/should_fail")
}

pub fn e2e_stdlib_dir() -> PathBuf {
    PathBuf::from("test/src/e2e_vm_tests/test_programs/should_pass/stdlib")
}

pub fn e2e_unit_dir() -> PathBuf {
    PathBuf::from("test/src/e2e_vm_tests/test_programs/should_pass/unit_tests")
}

pub fn e2e_test_dir() -> PathBuf {
    sway_workspace_dir()
        .join(e2e_language_dir())
        .join("struct_field_access")
}

pub fn std_lib_dir() -> PathBuf {
    sway_workspace_dir().join("sway-lib-std")
}

pub fn runnables_test_dir() -> PathBuf {
    test_fixtures_dir().join("runnables")
}

pub fn test_fixtures_dir() -> PathBuf {
    sway_workspace_dir().join("sway-lsp/tests/fixtures")
}

pub fn doc_comments_dir() -> PathBuf {
    sway_workspace_dir()
        .join(e2e_language_dir())
        .join("doc_comments")
}

pub fn generic_impl_self_dir() -> PathBuf {
    sway_workspace_dir()
        .join(e2e_language_dir())
        .join("generic_impl_self")
}

pub fn self_impl_reassignment_dir() -> PathBuf {
    sway_workspace_dir()
        .join(e2e_language_dir())
        .join("self_impl_reassignment")
}

pub fn get_absolute_path(path: &str) -> String {
    sway_workspace_dir().join(path).to_str().unwrap().into()
}

pub fn get_url(absolute_path: &str) -> Url {
    Url::parse(&format!("file://{}", &absolute_path)).expect("expected URL")
}

pub fn get_fixture(path: PathBuf) -> Value {
    let text = std::fs::read_to_string(path).expect("Failed to read file");
    serde_json::from_str::<Value>(&text).expect("Failed to parse JSON")
}

pub fn sway_example_dir() -> PathBuf {
    sway_workspace_dir().join("examples/storage_variables")
}

// Check if the given directory contains `Forc.toml` at its root.
pub fn dir_contains_forc_manifest(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().file_name().and_then(|s| s.to_str()) == Some("Forc.toml") {
                return true;
            }
        }
    }
    false
}

pub async fn assert_server_requests(
    socket: ClientSocket,
    expected_requests: Vec<Value>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let request_stream = socket.take(expected_requests.len()).collect::<Vec<_>>();
        let requests = request_stream.await;
        assert_eq!(requests.len(), expected_requests.len());
        for (actual, expected) in requests.iter().zip(expected_requests.iter()) {
            assert_eq!(expected["method"], actual.method());

            // Assert that all other expected fields are present without requiring
            // all actual fields to be present. Specifically we need this for `uri`,
            // which can't be hardcoded in the test.
            assert_json_include!(
                expected: expected,
                actual: serde_json::to_value(actual.clone()).unwrap()
            );
        }
    })
}

/// Introduces a random delay between 1 to 30 milliseconds with a chance of additional longer delays based on predefined probabilities.
pub async fn random_delay() {
    // Create a thread-safe RNG
    let mut rng = SmallRng::from_entropy();

    // wait for a random amount of time between 1-30ms
    tokio::time::sleep(tokio::time::Duration::from_millis(rng.gen_range(1..=30))).await;

    // 20% chance to introduce a longer delay of 100 to 1200 milliseconds.
    if rng.gen_ratio(2, 10) {
        tokio::time::sleep(tokio::time::Duration::from_millis(
            rng.gen_range(100..=1200),
        ))
        .await;
    }
}

/// Sets up the environment and a custom panic hook to print panic information and exit the program.
pub fn setup_panic_hook() {
    // Enable backtrace to get more information about panic
    std::env::set_var("RUST_BACKTRACE", "1");

    // Take the default panic hook
    let default_panic = std::panic::take_hook();

    // Set a custom panic hook
    std::panic::set_hook(Box::new(move |panic_info| {
        // Invoke the default panic hook to print the panic message
        default_panic(panic_info);
        std::process::exit(1);
    }));
}
