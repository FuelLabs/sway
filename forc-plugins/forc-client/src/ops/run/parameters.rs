#[derive(Debug)]
pub struct TxParameters {
    pub byte_price: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl TxParameters {
    pub const DEFAULT: Self = Self {
        gas_limit: fuel_tx::ConsensusParameters::DEFAULT.max_gas_per_tx,
        gas_price: 0,
    };

    pub fn new(gas_limit: Option<u64>, gas_price: Option<u64>) -> Self {
        Self {
            gas_limit: gas_limit.unwrap_or(TxParameters::DEFAULT.gas_limit),
            gas_price: gas_price.unwrap_or(TxParameters::DEFAULT.gas_price),
        }
    }
}

impl Default for TxParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}
