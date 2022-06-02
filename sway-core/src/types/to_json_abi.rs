use fuels_types::Property;

pub(crate) trait ToJsonAbi {
    fn generate_json_abi(&self) -> Option<Vec<Property>>;
}
