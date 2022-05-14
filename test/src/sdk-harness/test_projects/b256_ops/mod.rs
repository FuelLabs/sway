use test_helpers::script_runner;

#[tokio::test]
async fn b256_ops() {
    let path_to_bin = "test_projects/b256_ops/out/debug/b256_ops.bin";
    let return_val = script_runner(path_to_bin).await;
    assert_eq!(1, return_val);
}
