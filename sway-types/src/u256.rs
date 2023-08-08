use num_bigint::{BigUint, ParseBigIntError, TryFromBigIntError};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct U256(BigUint);

impl U256 {
    pub fn from_be_bytes(bytes: &[u8; 32]) -> Self {
        let v = BigUint::from_bytes_be(bytes.as_slice());
        Self(v)
    }

    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut v = self.0.to_bytes_be();
        let mut bytes = vec![0u8; 32 - v.len()];
        bytes.append(&mut v);
        assert!(bytes.len() == 32);
        bytes.try_into().expect("unexpected vector size")
    }
}

impl std::fmt::Display for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<BigUint> for U256 {
    fn from(value: BigUint) -> Self {
        Self(value)
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        Self(BigUint::from(value))
    }
}

impl TryFrom<U256> for u64 {
    type Error = Error;

    fn try_from(value: U256) -> Result<Self, Self::Error> {
        value.0.try_into().map_err(Error::TryIntoBigIntError)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    ParseBigIntError(ParseBigIntError),
    #[error("{0}")]
    TryIntoBigIntError(TryFromBigIntError<BigUint>),
}

impl std::str::FromStr for U256 {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = BigUint::from_str(s).map_err(Error::ParseBigIntError)?;
        Ok(Self(v))
    }
}
