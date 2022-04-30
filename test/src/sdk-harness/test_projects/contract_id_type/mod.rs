use test_helpers::script_runner;

#[tokio::test]
async fn contract_id_eq_implementation() {
    let path_to_bin = "test_projects/contract_id_type/out/debug/contract_id_type.bin";
    let return_val = script_runner(path_to_bin).await;
    assert_eq!(1, return_val);
}
