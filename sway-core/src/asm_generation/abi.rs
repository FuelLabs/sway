use super::EvmAbiResult;

#[derive(Clone, Debug)]
pub enum ProgramABI {
    Fuel(fuels_types::ProgramABI),
    Evm(EvmAbiResult),
}
