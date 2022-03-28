#[derive(Debug)]
pub(crate) struct TxParameters {
    pub byte_price: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl TxParameters {
    pub const DEFAULT: Self = Self {
        byte_price: 0,
        gas_limit: fuel_tx::consts::MAX_GAS_PER_TX,
        gas_price: 0,
    };

    pub fn new(byte_price: Option<u64>, gas_limit: Option<u64>, gas_price: Option<u64>) -> Self {
        Self {
            byte_price: byte_price.unwrap_or(TxParameters::DEFAULT.byte_price),
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
