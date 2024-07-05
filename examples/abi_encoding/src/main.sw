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
