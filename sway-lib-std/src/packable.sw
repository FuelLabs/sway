library packable;

use ::bytes::Bytes;

pub trait Packable {
    fn pack(self) -> Bytes;
    // fn unpack(bytes: Bytes) -> Self;
}

