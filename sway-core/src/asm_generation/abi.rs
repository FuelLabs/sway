use super::EvmAbiResult;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ProgramABI {
    Fuel(fuel_abi_types::abi::program::ProgramABI),
    Evm(EvmAbiResult),
    MidenVM(()),
}
