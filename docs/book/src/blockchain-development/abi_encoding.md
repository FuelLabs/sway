# ABI Encoding

Application binary interface (ABI) encoding typically enables programs to communicate with each other with the same data encoding system.

The Sway language provides helpful traits and utilities to help with [Fuel ABI Encoding](https://docs.fuel.network/docs/specs/abi/) within the language, which is used across Sway programs.

Sway, at its core, is agnostic to ABI encoding but preferences the Fuel ABI Encoding format.

## ABI encoding with the `abi_encode` and `abi_decode` function
This function will encode a structure into an ABI encoded bytes vector.

All primitive and complex types have an `abi_encode` and `abi_decode` method.

## Example
```sway
script;

fn main() {
    let value = u256::max();
    
    // ABI Encode
    let buffer = Buffer::new();
    value.abi_encode(buffer);
    
    // ABI Decode
    let slice = buffer.as_raw_slice();
    let mut reader = BufferReader::from_parts(slice.ptr(), slice.number_of_bytes());
    let decoded_u256 = u256::abi_decode(reader);
    
    assert(value == decoded_u256);
}
```
