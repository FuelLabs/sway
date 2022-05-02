use test_helpers::script_runner;

#[tokio::test]
async fn can_perform_exponentiation() {
    let path_to_bin = "test_projects/exponentiation/out/debug/exponentiation.bin";
    let return_val = script_runner(path_to_bin).await;
    assert_eq!(return_val, 1);
}
