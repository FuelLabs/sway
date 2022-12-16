library contract_id;
//! A wrapper around the `b256` type to help enhance type-safety.
use ::assert::assert;
use ::bytes::Bytes;
use ::convert::From;
use ::option::Option;
use ::packable::Packable;

/// The `ContractId` type, a struct wrappper around the inner `b256` value.
pub struct ContractId {
    value: b256,
}

impl core::ops::Eq for ContractId {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `ContractId` types.
impl From<b256> for ContractId {
    fn from(bits: b256) -> ContractId {
        ContractId { value: bits }
    }

    fn into(self) -> b256 {
        self.value
    }
}

/// functions for converting between the ContractId and Bytes types.
impl Packable for ContractId {
    fn pack(self) -> Bytes {
        // Artificially create bytes with capacity and len
        let mut bytes = Bytes::with_capacity(32);
        bytes.len = 32;

        // Copy bytes from contract_id into the buffer of the target bytes
        __addr_of(self).copy_bytes_to(bytes.buf.ptr, 32);

        bytes
    }

    fn unpack(bytes: Bytes) -> Self {
        let mut new_id = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let ptr = __addr_of(new_id);
        bytes.buf.ptr().copy_to::<b256>(ptr, 1);

        ContractId::from(new_id)
    }
}

#[test]
fn test_pack() {
    let initial = 0x3333333333333333333333333333333333333333333333333333333333333333;
    let id = ContractId::from(initial);
    let packed = id.pack();
    let mut control_bytes = Bytes::with_capacity(32);
    // 0x33 is 51 in decimal
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);
    control_bytes.push(51u8);

    assert(packed == control_bytes);
}

#[test]
fn test_unpack() {
    let mut initial_bytes = Bytes::with_capacity(32);
    // 0x33 is 51 in decimal
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);

    let resulting_id = ContractId::unpack(initial_bytes);
    let expected_id = ContractId::from(0x3333333333333333333333333333333333333333333333333333333333333333);

    assert(resulting_id == expected_id);
}
