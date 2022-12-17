library pack;

use ::bytes::Bytes;

pub trait Pack {
    fn pack(self) -> Bytes;
    fn unpack(bytes: Bytes) -> Self;
}
