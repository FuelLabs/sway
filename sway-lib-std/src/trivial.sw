library;

use ::codec::{AbiDecode, AbiEncode, Buffer, BufferReader};
use ::marker::Enum;

pub struct TrivialBool {
    value: u8,
}

impl TrivialBool {
    pub fn unwrap(self) -> bool {
        match self.value {
            0 => false,
            1 => true,
            _ => __revert(0),
        }
    }
}

impl AbiEncode for TrivialBool {
    // fn is_encode_trivial() -> bool {
    //    true
    // }

    fn abi_encode(self, buffer: Buffer) -> Buffer {
        self.value.abi_encode(buffer)
    }
}

impl AbiDecode for TrivialBool {
    // fn is_decode_trivial() -> bool {
    //    true
    // }

    fn abi_decode(ref mut buffer: BufferReader) -> TrivialBool {
        let value: u8 = buffer.read::<u8>();
        TrivialBool { value }
    }
}

pub struct TrivialEnum<T>
where
    T: Enum + AbiEncode + AbiDecode,
{
    value: T,
}

impl<T> TrivialEnum<T>
where
    T: Enum + AbiEncode + AbiDecode,
{
    pub fn unwrap(self) -> T {
        // this is not useless, as an invalid
        // discriminant will revert
        match self.value {
            v => v,
        }
    }
}

impl<T> AbiEncode for TrivialEnum<T>
where
    T: Enum + AbiEncode + AbiDecode,
{
    // fn is_encode_trivial() -> bool {
    //    true
    // }

    fn abi_encode(self, buffer: Buffer) -> Buffer {
        self.value.abi_encode(buffer)
    }
}

impl<T> AbiDecode for TrivialEnum<T>
where
    T: Enum + AbiEncode + AbiDecode,
{
    // fn is_decode_trivial() -> bool {
    //    true
    // }

    fn abi_decode(ref mut buffer: BufferReader) -> TrivialEnum<T> {
        let value: T = buffer.decode::<T>();
        TrivialEnum { value }
    }
}
