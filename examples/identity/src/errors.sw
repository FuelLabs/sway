library;

use core::codec::*;

pub enum MyError {
    UnauthorizedUser: Identity,
}

impl AbiEncode for MyError {
    fn abi_encode(self, ref mut buffer: Buffer) {
        match self {
            MyError::UnauthorizedUser(identity) => {
                identity.abi_encode(buffer);
            }
        }
    }
}
