library r#abi;

abi IdentityExample {
    #[storage(read)]
    fn access_control_with_identity();
    fn cast_to_identity();
    fn different_executions(my_identity: Identity);
    fn identity_to_contract_id(my_identity: Identity);
}
