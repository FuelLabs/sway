pub trait ToJsonAbi {
    type Output;

    fn generate_json_abi(&self) -> Self::Output;
}
