//! A value representing a storage key. Every storage field access in the program
//! corresponds to a [StorageKey].

use std::vec;

use crate::{
    context::Context, irtype::Type, pretty::DebugWithContext, Constant, ConstantContent,
    ConstantValue, B256,
};

/// A wrapper around an [ECS](https://github.com/orlp/slotmap) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, DebugWithContext)]
pub struct StorageKey(#[in_context(storage_keys)] pub slotmap::DefaultKey);

#[doc(hidden)]
#[derive(Clone, DebugWithContext)]
pub struct StorageKeyContent {
    pub ptr_ty: Type,
    pub key: Constant,
}

impl StorageKey {
    pub fn new(context: &mut Context, slot: [u8; 32], offset: u64, field_id: [u8; 32]) -> Self {
        // Construct `ptr { b256, u64, b256 }`.
        let b256_ty = Type::get_b256(context);
        let uint64_ty = Type::get_uint64(context);

        let key_ty = Type::new_struct(context, vec![b256_ty, uint64_ty, b256_ty]);
        let ptr_ty = Type::new_typed_pointer(context, key_ty);

        let slot = ConstantContent::new_b256(context, slot);
        let offset = ConstantContent::new_uint(context, 64, offset);
        let field_id = ConstantContent::new_b256(context, field_id);

        let key = ConstantContent::new_struct(
            context,
            vec![b256_ty, uint64_ty, b256_ty],
            vec![slot, offset, field_id],
        );

        let key = Constant::unique(context, key);

        let content = StorageKeyContent { ptr_ty, key };

        StorageKey(context.storage_keys.insert(content))
    }

    /// Return the storage key type, which is always `ptr { b256, u64, b256 }`.
    pub fn get_type(&self, context: &Context) -> Type {
        context.storage_keys[self.0].ptr_ty
    }

    /// Return the storage key, which is a constant of type `{ b256, u64, b256 }`.
    pub fn get_key(&self, context: &Context) -> Constant {
        context.storage_keys[self.0].key
    }

    /// Return the three parts of this storage key: `(slot, offset, field_id)`.
    pub fn get_parts<'a>(&self, context: &'a Context) -> (&'a B256, u64, &'a B256) {
        let ConstantContent {
            value: ConstantValue::Struct(fields),
            ..
        } = &context.storage_keys[self.0].key.get_content(context)
        else {
            unreachable!("`StorageKey::key` constant content is a struct with three fields");
        };

        let ConstantContent {
            value: ConstantValue::B256(slot),
            ..
        } = &fields[0]
        else {
            unreachable!("storage key slot is a `B256` constant");
        };
        let ConstantContent {
            value: ConstantValue::Uint(offset),
            ..
        } = &fields[1]
        else {
            unreachable!("storage key offset is a `u64` constant");
        };
        let ConstantContent {
            value: ConstantValue::B256(field_id),
            ..
        } = &fields[2]
        else {
            unreachable!("storage key field_id is a `B256` constant");
        };

        (slot, *offset, field_id)
    }
}
