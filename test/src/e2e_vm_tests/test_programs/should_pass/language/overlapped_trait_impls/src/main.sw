script;

use std::{
    bytes::Bytes,
    vec::Vec,
};

pub trait FromBytesConvertible {
    fn _from_be_bytes(bytes: Bytes) -> Self;
}

pub trait FromBytes {
    fn from_bytes(bytes: Bytes) -> Self;
}

impl<T> FromBytes for T
where
    T: FromBytesConvertible,
{
    fn from_bytes(bytes: Bytes) -> Self {
        Self::_from_be_bytes(bytes)
    }
}

pub struct DataPoint {}
pub struct Payload {}

impl FromBytes for DataPoint {
    fn from_bytes(bytes: Bytes) -> Self {
        Self {}
    }
}

impl Payload {
    pub fn from_bytes(bytes: Bytes) {
        let mut data_points = Vec::new();

        data_points.push(DataPoint::from_bytes(bytes));

        let a:DataPoint = DataPoint::from_bytes(bytes);
        data_points.push(a);
    }
}

pub fn main() -> bool {
    Payload::from_bytes(Bytes::new());

    true
}