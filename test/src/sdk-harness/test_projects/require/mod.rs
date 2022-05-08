use test_helpers::script_runner;

#[tokio::test]
async fn require_function() {
    let path_to_bin = "test_projects/require/out/debug/require.bin";
    let return_val = script_runner(path_to_bin).await;
    assert_eq!(1, return_val);
}
