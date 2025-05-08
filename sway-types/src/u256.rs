use num_bigint::{BigUint, ParseBigIntError, TryFromBigIntError};
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::ops::{Not, Shl, Shr};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize, Deserialize)]
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

    pub fn checked_add(&self, other: &U256) -> Option<U256> {
        let r = &self.0 + &other.0;
        (r.bits() <= 256).then_some(Self(r))
    }

    pub fn checked_sub(&self, other: &U256) -> Option<U256> {
        (self.0 >= other.0).then(|| Self(&self.0 - &other.0))
    }

    pub fn checked_mul(&self, other: &U256) -> Option<U256> {
        let r = &self.0 * &other.0;
        (r.bits() <= 256).then_some(Self(r))
    }

    pub fn checked_div(&self, other: &U256) -> Option<U256> {
        other.0.is_zero().not().then(|| Self(&self.0 / &other.0))
    }

    pub fn shr(&self, other: &u64) -> U256 {
        U256((&self.0).shr(other))
    }

    pub fn checked_shl(&self, other: &u64) -> Option<U256> {
        let r = (&self.0).shl(other);
        (r.bits() <= 256).then_some(Self(r))
    }

    pub fn checked_rem(&self, other: &U256) -> Option<U256> {
        if other.0 == BigUint::ZERO {
            None
        } else {
            Some(U256(&self.0 % &other.0))
        }
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

impl<'a> std::ops::BitAnd<&'a U256> for &'a U256 {
    type Output = U256;

    fn bitand(self, rhs: Self) -> Self::Output {
        U256((&self.0).bitand(&rhs.0))
    }
}

impl<'a> std::ops::BitOr<&'a U256> for &'a U256 {
    type Output = U256;

    fn bitor(self, rhs: Self) -> Self::Output {
        U256((&self.0).bitor(&rhs.0))
    }
}

impl<'a> std::ops::BitXor<&'a U256> for &'a U256 {
    type Output = U256;

    fn bitxor(self, rhs: Self) -> Self::Output {
        U256((&self.0).bitxor(&rhs.0))
    }
}

impl<'a> std::ops::Rem<&'a U256> for &'a U256 {
    type Output = U256;

    fn rem(self, rhs: Self) -> Self::Output {
        U256((&self.0).rem(&rhs.0))
    }
}

impl std::ops::Not for &U256 {
    type Output = U256;

    fn not(self) -> Self::Output {
        let mut bytes = self.to_be_bytes();
        bytes.iter_mut().for_each(|b| *b = !*b);
        U256(BigUint::from_bytes_be(&bytes))
    }
}
