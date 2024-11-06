use crate::fuel_prelude::{
    fuel_crypto::Hasher,
    fuel_tx::StorageSlot,
    fuel_types::{Bytes32, Bytes8},
};
use sway_features::ExperimentalFeatures;
use sway_ir::{
    constant::{Constant, ConstantValue},
    context::Context,
    irtype::Type,
};
use sway_types::u256::U256;

/// Determines how values that are less then a word in length
/// has to be padded to word boundary when in structs or enums.
#[derive(Default)]
enum InByte8Padding {
    #[default]
    Right,
    Left,
}

/// Hands out storage keys using storage field names or an existing key.
/// Basically returns sha256((0u8, "storage::<storage_namespace_name1>::<storage_namespace_name2>.<storage_field_name>"))
/// or key if defined.
pub(super) fn get_storage_key(storage_field_names: Vec<String>, key: Option<U256>, experimental: ExperimentalFeatures) -> Bytes32 {
    match key {
        Some(key) => key.to_be_bytes().into(),
        None => hash_storage_key_string(get_storage_key_string(&storage_field_names), experimental),
    }
}

pub fn get_storage_key_string(storage_field_names: &[String]) -> String {
    if storage_field_names.len() == 1 {
        format!(
            "{}{}{}",
            sway_utils::constants::STORAGE_TOP_LEVEL_NAMESPACE,
            sway_utils::constants::STORAGE_FIELD_SEPARATOR,
            storage_field_names.last().unwrap(),
        )
    } else {
        format!(
            "{}{}{}{}{}",
            sway_utils::constants::STORAGE_TOP_LEVEL_NAMESPACE,
            sway_utils::constants::STORAGE_NAMESPACE_SEPARATOR,
            storage_field_names
                .iter()
                .take(storage_field_names.len() - 1)
                .cloned()
                .collect::<Vec<_>>()
                .join(sway_utils::constants::STORAGE_NAMESPACE_SEPARATOR),
            sway_utils::constants::STORAGE_FIELD_SEPARATOR,
            storage_field_names.last().unwrap(),
        )
    }
}

/// Hands out unique storage field ids using storage field names and struct field names.
/// Basically returns sha256((0u8, "storage::<storage_namespace_name1>::<storage_namespace_name2>.<storage_field_name>.<struct_field_name1>.<struct_field_name2>")).
pub(super) fn get_storage_field_id(
    storage_field_names: &[String],
    struct_field_names: &[String],
    experimental: ExperimentalFeatures,
) -> Bytes32 {
    let data = format!(
        "{}{}",
        get_storage_key_string(storage_field_names),
        if struct_field_names.is_empty() {
            "".to_string()
        } else {
            format!(
                "{}{}",
                sway_utils::constants::STRUCT_FIELD_SEPARATOR,
                struct_field_names.join(sway_utils::constants::STRUCT_FIELD_SEPARATOR),
            )
        }
    );

    hash_storage_key_string(data, experimental)
}

fn hash_storage_key_string(storage_key_string: String, experimental: ExperimentalFeatures) -> Bytes32 {
    let mut hasher = Hasher::default();
    // Certain storage types, like, e.g., `StorageMap` allow
    // storage slots of their contained elements to be defined
    // based on developer's input. E.g., the `key` in a `StorageMap`
    // used to calculate the storage slot is a developer input.
    //
    // To ensure that pre-images of such storage slots can never
    // be the same as a pre-image of compiler generated key of storage
    // field, we prefix the pre-images with a single byte that denotes
    // the domain. Storage types like `StorageMap` must have a different
    // domain prefix than the `STORAGE_DOMAIN` which is 0u8.
    //
    // For detailed elaboration see: https://github.com/FuelLabs/sway/issues/6317
    if experimental.storage_domains {
        hasher.input(sway_utils::constants::STORAGE_DOMAIN);
    }
    hasher.input(storage_key_string);
    hasher.finalize()
}

use uint::construct_uint;

#[allow(
// These two warnings are generated by the `construct_uint!()` macro below.
    clippy::assign_op_pattern,
    clippy::ptr_offset_with_cast
)]
pub(super) fn add_to_b256(x: Bytes32, y: u64) -> Bytes32 {
    construct_uint! {
        struct U256(4);
    }
    let x = U256::from(*x);
    let y = U256::from(y);
    let res: [u8; 32] = (x + y).into();
    Bytes32::from(res)
}

/// Given a constant value `constant`, a type `ty`, a state index, and a vector of subfield
/// indices, serialize the constant into a vector of storage slots. The keys (slots) are
/// generated using the state index and the subfield indices which are recursively built. The
/// values are generated such that each subfield gets its own storage slot except for enums and
/// strings which are spread over successive storage slots (use `serialize_to_words` in this case).
///
/// This behavior matches the behavior of how storage slots are assigned for storage reads and
/// writes (i.e. how `state_read_*` and `state_write_*` instructions are generated).
pub fn serialize_to_storage_slots(
    constant: &Constant,
    context: &Context,
    storage_field_names: Vec<String>,
    key: Option<U256>,
    ty: &Type,
) -> Vec<StorageSlot> {
    let experimental = context.experimental;
    match &constant.value {
        ConstantValue::Undef => vec![],
        // If not being a part of an aggregate, single byte values like `bool`, `u8`, and unit
        // are stored as a byte at the beginning of the storage slot.
        ConstantValue::Unit if ty.is_unit(context) => vec![StorageSlot::new(
            get_storage_key(storage_field_names, key, experimental),
            Bytes32::new([0; 32]),
        )],
        ConstantValue::Bool(b) if ty.is_bool(context) => {
            vec![StorageSlot::new(
                get_storage_key(storage_field_names, key, experimental),
                Bytes32::new([
                    if *b { 1 } else { 0 },
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]),
            )]
        }
        ConstantValue::Uint(b) if ty.is_uint8(context) => {
            vec![StorageSlot::new(
                get_storage_key(storage_field_names, key, experimental),
                Bytes32::new([
                    *b as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0,
                ]),
            )]
        }
        // Similarly, other uint values are stored at the beginning of the storage slot.
        ConstantValue::Uint(n) if ty.is_uint(context) => {
            vec![StorageSlot::new(
                get_storage_key(storage_field_names, key, experimental),
                Bytes32::new(
                    n.to_be_bytes()
                        .iter()
                        .cloned()
                        .chain([0; 24].iter().cloned())
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap(),
                ),
            )]
        }
        ConstantValue::U256(b) if ty.is_uint_of(context, 256) => {
            vec![StorageSlot::new(
                get_storage_key(storage_field_names, key, experimental),
                Bytes32::new(b.to_be_bytes()),
            )]
        }
        ConstantValue::B256(b) if ty.is_b256(context) => {
            vec![StorageSlot::new(
                get_storage_key(storage_field_names, key, experimental),
                Bytes32::new(b.to_be_bytes()),
            )]
        }
        ConstantValue::Array(_a) if ty.is_array(context) => {
            unimplemented!("Arrays in storage have not been implemented yet.")
        }
        _ if ty.is_string_array(context) || ty.is_struct(context) || ty.is_union(context) => {
            // Serialize the constant data in words and add zero words until the number of words
            // is a multiple of 4. This is useful because each storage slot is 4 words.
            // Regarding padding, the top level type in the call is either a string array, struct, or
            // a union. They will properly set the initial padding for the further recursive calls.
            let mut packed = serialize_to_words(constant, context, ty, InByte8Padding::default());
            packed.extend(vec![
                Bytes8::new([0; 8]);
                ((packed.len() + 3) / 4) * 4 - packed.len()
            ]);

            assert!(packed.len() % 4 == 0);

            // Return a list of `StorageSlot`s
            // First get the keys then get the values
            // TODO-MEMLAY: Warning! Here we make an assumption about the memory layout of
            //       string arrays, structs, and enum.
            //       The assumption is that they are rounded to word boundaries
            //       which will very likely always be the case.
            //       We will not refactor the Storage API at the moment to remove this
            //       assumption. It is a questionable effort because we anyhow
            //       want to improve and refactor Storage API in the future.
            let type_size_in_bytes = ty.size(context).in_bytes();
            assert!(
                type_size_in_bytes % 8 == 0,
                "Expected string arrays, structs, and enums to be aligned to word boundary. The type size in bytes was {} and the type was {}.",
                type_size_in_bytes,
                ty.as_string(context)
            );

            let storage_key = get_storage_key(storage_field_names, key, experimental);
            (0..(type_size_in_bytes + 31) / 32)
                .map(|i| add_to_b256(storage_key, i))
                .zip((0..packed.len() / 4).map(|i| {
                    Bytes32::new(
                        Vec::from_iter((0..4).flat_map(|j| *packed[4 * i + j]))
                            .try_into()
                            .unwrap(),
                    )
                }))
                .map(|(k, r)| StorageSlot::new(k, r))
                .collect()
        }
        _ => vec![],
    }
}

/// Given a constant value `constant` and a type `ty`, serialize the constant into a vector of
/// words and apply the requested padding if needed.
fn serialize_to_words(
    constant: &Constant,
    context: &Context,
    ty: &Type,
    padding: InByte8Padding,
) -> Vec<Bytes8> {
    match &constant.value {
        ConstantValue::Undef => vec![],
        ConstantValue::Unit if ty.is_unit(context) => vec![Bytes8::new([0; 8])],
        ConstantValue::Bool(b) if ty.is_bool(context) => match padding {
            InByte8Padding::Right => {
                vec![Bytes8::new([if *b { 1 } else { 0 }, 0, 0, 0, 0, 0, 0, 0])]
            }
            InByte8Padding::Left => {
                vec![Bytes8::new([0, 0, 0, 0, 0, 0, 0, if *b { 1 } else { 0 }])]
            }
        },
        ConstantValue::Uint(n) if ty.is_uint8(context) => match padding {
            InByte8Padding::Right => vec![Bytes8::new([*n as u8, 0, 0, 0, 0, 0, 0, 0])],
            InByte8Padding::Left => vec![Bytes8::new([0, 0, 0, 0, 0, 0, 0, *n as u8])],
        },
        ConstantValue::Uint(n) if ty.is_uint(context) => {
            vec![Bytes8::new(n.to_be_bytes())]
        }
        ConstantValue::U256(b) if ty.is_uint_of(context, 256) => {
            let b = b.to_be_bytes();
            Vec::from_iter((0..4).map(|i| Bytes8::new(b[8 * i..8 * i + 8].try_into().unwrap())))
        }
        ConstantValue::B256(b) if ty.is_b256(context) => {
            let b = b.to_be_bytes();
            Vec::from_iter((0..4).map(|i| Bytes8::new(b[8 * i..8 * i + 8].try_into().unwrap())))
        }
        ConstantValue::String(s) if ty.is_string_array(context) => {
            // Turn the bytes into serialized words (Bytes8) and right pad it to the word boundary.
            let mut s = s.clone();
            s.extend(vec![0; ((s.len() + 7) / 8) * 8 - s.len()]);

            assert!(s.len() % 8 == 0);

            // Group into words.
            Vec::from_iter((0..s.len() / 8).map(|i| {
                Bytes8::new(
                    Vec::from_iter((0..8).map(|j| s[8 * i + j]))
                        .try_into()
                        .unwrap(),
                )
            }))
        }
        ConstantValue::Array(_) if ty.is_array(context) => {
            unimplemented!("Arrays in storage have not been implemented yet.")
        }
        ConstantValue::Struct(vec) if ty.is_struct(context) => {
            let field_tys = ty.get_field_types(context);
            vec.iter()
                .zip(field_tys.iter())
                // TODO-MEMLAY: Warning! Again, making an assumption about the memory layout
                //       of struct fields.
                .flat_map(|(f, ty)| serialize_to_words(f, context, ty, InByte8Padding::Right))
                .collect()
        }
        _ if ty.is_union(context) => {
            let value_size_in_words = ty.size(context).in_words();
            let constant_size_in_words = constant.ty.size(context).in_words();
            assert!(value_size_in_words >= constant_size_in_words);

            // Add enough left padding to satisfy the actual size of the union
            // TODO-MEMLAY: Warning! Here we make an assumption about the memory layout of enums,
            //       that they are left padded.
            //       The memory layout of enums can be changed in the future.
            //       We will not refactor the Storage API at the moment to remove this
            //       assumption. It is a questionable effort because we anyhow
            //       want to improve and refactor Storage API in the future.
            let padding_size_in_words = value_size_in_words - constant_size_in_words;
            vec![Bytes8::new([0; 8]); padding_size_in_words as usize]
                .iter()
                .cloned()
                .chain(
                    serialize_to_words(constant, context, &constant.ty, InByte8Padding::Left)
                        .iter()
                        .cloned(),
                )
                .collect()
        }
        _ => vec![],
    }
}
