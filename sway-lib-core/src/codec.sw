library;

pub trait AbiEncode {
    fn abi_encode(self);
}

pub fn encode<T>(value: T)
where
    T: AbiEncode
{
    value.abi_encode();
}
