pub const DEFAULT_BYTE_PRICE: u64 = 0;
pub const DEFAULT_GAS_LIMIT: u64 = fuel_tx::consts::MAX_GAS_PER_TX;
pub const DEFAULT_GAS_PRICE: u64 = 0;

#[derive(Debug)]
pub struct TxParameters {
    pub byte_price: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl Default for TxParameters {
    fn default() -> Self {
        Self {
            byte_price: DEFAULT_BYTE_PRICE,
            gas_limit: DEFAULT_GAS_LIMIT,
            gas_price: DEFAULT_GAS_PRICE,
        }
    }
}

impl TxParameters {
    pub fn new(byte_price: Option<u64>, gas_limit: Option<u64>, gas_price: Option<u64>) -> Self {
        Self {
            byte_price: byte_price.unwrap_or(DEFAULT_BYTE_PRICE),
            gas_limit: gas_limit.unwrap_or(DEFAULT_GAS_LIMIT),
            gas_price: gas_price.unwrap_or(DEFAULT_GAS_PRICE),
        }
    }
}
