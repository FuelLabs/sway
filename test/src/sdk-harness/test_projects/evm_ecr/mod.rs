use test_helpers::script_runner;

#[tokio::test]
async fn evm_ecr_implementation() {
    let path_to_bin = "test_projects/evm_ecr/out/debug/evm_ecr.bin";
    let return_val = script_runner(path_to_bin).await;
    assert_eq!(1, return_val);
}
