use super::EvmAbiResult;

#[derive(Clone, Debug)]
pub enum ProgramABI {
    Fuel(fuel_abi_types::program_abi::ProgramABI),
    Evm(EvmAbiResult),
}
