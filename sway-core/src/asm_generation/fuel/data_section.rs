use rustc_hash::{FxHashMap, FxHashSet};
use sway_ir::{
    size_bytes_round_up_to_word_alignment, ConstantContent, ConstantValue, Context, Padding,
};

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum EntryName {
    NonConfigurable,
    Configurable(String),
}

impl fmt::Display for EntryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryName::NonConfigurable => write!(f, "NonConfigurable"),
            EntryName::Configurable(name) => write!(f, "<Configurable, {name}>"),
        }
    }
}

// An entry in the data section.  It's important for the size to be correct, especially for unions
// where the size could be larger than the represented value.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Entry {
    pub value: Datum,
    pub padding: Padding,
    pub name: EntryName,
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum Datum {
    Byte(u8),
    Word(u64),
    ByteArray(Vec<u8>),
    Slice(Vec<u8>),
    Collection(Vec<Entry>),
}

impl Entry {
    pub(crate) fn new_byte(value: u8, name: EntryName, padding: Option<Padding>) -> Entry {
        Entry {
            value: Datum::Byte(value),
            padding: padding.unwrap_or(Padding::default_for_u8(value)),
            name,
        }
    }

    pub(crate) fn new_word(value: u64, name: EntryName, padding: Option<Padding>) -> Entry {
        Entry {
            value: Datum::Word(value),
            padding: padding.unwrap_or(Padding::default_for_u64(value)),
            name,
        }
    }

    pub(crate) fn new_byte_array(
        bytes: Vec<u8>,
        name: EntryName,
        padding: Option<Padding>,
    ) -> Entry {
        Entry {
            padding: padding.unwrap_or(Padding::default_for_byte_array(&bytes)),
            value: Datum::ByteArray(bytes),
            name,
        }
    }

    pub(crate) fn new_slice(bytes: Vec<u8>, name: EntryName, padding: Option<Padding>) -> Entry {
        Entry {
            padding: padding.unwrap_or(Padding::default_for_byte_array(&bytes)),
            value: Datum::Slice(bytes),
            name,
        }
    }

    pub(crate) fn new_collection(
        elements: Vec<Entry>,
        name: EntryName,
        padding: Option<Padding>,
    ) -> Entry {
        Entry {
            padding: padding.unwrap_or(Padding::default_for_aggregate(
                elements.iter().map(|el| el.padding.target_size()).sum(),
            )),
            value: Datum::Collection(elements),
            name,
        }
    }

    pub(crate) fn from_constant(
        context: &Context,
        constant: &ConstantContent,
        name: EntryName,
        padding: Option<Padding>,
    ) -> Entry {
        // We need a special handling in case of enums.
        if constant.ty.is_enum(context) {
            let (tag, value) = constant
                .enum_tag_and_value_with_paddings(context)
                .expect("Constant is an enum.");

            let tag_entry = Entry::from_constant(context, tag.0, EntryName::NonConfigurable, tag.1);
            let value_entry =
                Entry::from_constant(context, value.0, EntryName::NonConfigurable, value.1);

            return Entry::new_collection(vec![tag_entry, value_entry], name, padding);
        }

        // Not an enum, no more special handling required.
        match &constant.value {
            ConstantValue::Undef | ConstantValue::Unit => Entry::new_byte(0, name, padding),
            ConstantValue::Bool(value) => Entry::new_byte(u8::from(*value), name, padding),
            ConstantValue::Uint(value) => {
                if constant.ty.is_uint8(context) {
                    Entry::new_byte(*value as u8, name, padding)
                } else {
                    Entry::new_word(*value, name, padding)
                }
            }
            ConstantValue::U256(value) => {
                Entry::new_byte_array(value.to_be_bytes().to_vec(), name, padding)
            }
            ConstantValue::B256(value) => {
                Entry::new_byte_array(value.to_be_bytes().to_vec(), name, padding)
            }
            ConstantValue::String(bytes) => Entry::new_byte_array(bytes.clone(), name, padding),
            ConstantValue::Array(_) => Entry::new_collection(
                constant
                    .array_elements_with_padding(context)
                    .expect("Constant is an array.")
                    .into_iter()
                    .map(|(elem, padding)| {
                        Entry::from_constant(context, elem, EntryName::NonConfigurable, padding)
                    })
                    .collect(),
                name,
                padding,
            ),
            ConstantValue::Struct(_) => Entry::new_collection(
                constant
                    .struct_fields_with_padding(context)
                    .expect("Constant is a struct.")
                    .into_iter()
                    .map(|(elem, padding)| {
                        Entry::from_constant(context, elem, EntryName::NonConfigurable, padding)
                    })
                    .collect(),
                name,
                padding,
            ),
            ConstantValue::RawUntypedSlice(bytes) => Entry::new_slice(bytes.clone(), name, padding),
            ConstantValue::Reference(_) => {
                todo!("Constant references are currently not supported.")
            }
            ConstantValue::Slice(_) => {
                todo!("Constant slices are currently not supported.")
            }
        }
    }

    /// Converts a literal to a big-endian representation. This is padded to words.
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        // Get the big-endian byte representation of the basic value.
        let bytes = match &self.value {
            Datum::Byte(value) => vec![*value],
            Datum::Word(value) => value.to_be_bytes().to_vec(),
            Datum::ByteArray(bytes) | Datum::Slice(bytes) if bytes.len() % 8 == 0 => bytes.clone(),
            Datum::ByteArray(bytes) | Datum::Slice(bytes) => bytes
                .iter()
                .chain([0; 8].iter())
                .copied()
                .take((bytes.len() + 7) & 0xfffffff8_usize)
                .collect(),
            Datum::Collection(items) => items.iter().flat_map(|el| el.to_bytes()).collect(),
        };

        let final_padding = self.padding.target_size().saturating_sub(bytes.len());
        match self.padding {
            Padding::Left { .. } => {
                [std::iter::repeat_n(0u8, final_padding).collect(), bytes].concat()
            }
            Padding::Right { .. } => {
                [bytes, std::iter::repeat_n(0u8, final_padding).collect()].concat()
            }
        }
    }

    pub(crate) fn has_copy_type(&self) -> bool {
        matches!(self.value, Datum::Word(_) | Datum::Byte(_))
    }

    pub(crate) fn is_byte(&self) -> bool {
        matches!(self.value, Datum::Byte(_))
    }

    /// A short, human-readable description of this entry's value (for ASM
    /// dumps). Mirrors the format used by the `DataSection` `.data:` dump so
    /// the pool dump and the data-section dump read the same.
    pub(crate) fn display_value(&self) -> String {
        fn display_bytes(bs: &[u8]) -> String {
            let mut hex = String::new();
            let mut chr = String::new();
            for b in bs {
                hex.push_str(format!("{b:02x} ").as_str());
                chr.push(if *b == b' ' || b.is_ascii_graphic() {
                    *b as char
                } else {
                    '.'
                });
            }
            format!("[{}] {hex} {chr}", bs.len())
        }
        match &self.value {
            Datum::Byte(w) => format!(".byte {w}"),
            Datum::Word(w) => format!(".word {w}"),
            Datum::ByteArray(bs) => format!(".bytes{}", display_bytes(bs)),
            Datum::Slice(bs) => format!(".slice{}", display_bytes(bs)),
            Datum::Collection(els) => format!(
                ".collection {{ {} }}",
                els.iter()
                    .map(|el| el.display_value())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    pub(crate) fn equiv(&self, entry: &Entry) -> bool {
        fn equiv_data(lhs: &Datum, rhs: &Datum) -> bool {
            match (lhs, rhs) {
                (Datum::Byte(l), Datum::Byte(r)) => l == r,
                (Datum::Word(l), Datum::Word(r)) => l == r,
                (Datum::ByteArray(l), Datum::ByteArray(r)) => l == r,
                (Datum::Collection(l), Datum::Collection(r)) => {
                    l.len() == r.len()
                        && l.iter()
                            .zip(r.iter())
                            .all(|(l, r)| equiv_data(&l.value, &r.value))
                }
                _ => false,
            }
        }

        // If this corresponds to a configuration-time constants, then the entry names will be
        // available (i.e. `Some(..)`) and they must be the same before we can merge the two
        // entries. Otherwise, `self.name` and `entry.name` will be `None` in which case we're also
        // allowed to merge the two entries (if their values are equivalent of course).
        equiv_data(&self.value, &entry.value) && self.name == entry.name
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataIdEntryKind {
    NonConfigurable,
    Configurable,
}

impl fmt::Display for DataIdEntryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataIdEntryKind::NonConfigurable => write!(f, "NonConfigurable"),
            DataIdEntryKind::Configurable => write!(f, "Configurable"),
        }
    }
}

/// An address which refers to a value in the data section of the asm.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DataId {
    pub(crate) idx: u32,
    pub(crate) kind: DataIdEntryKind,
}

impl fmt::Display for DataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data_{}_{}", self.kind, self.idx)
    }
}

/// An index which refers to a value in a function's literal pool.
///
/// Unlike [`DataId`], a `PoolEntryId` is only meaningful within the single
/// function whose [`crate::asm_lang::VirtualOp::LiteralPool`] it indexes into.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PoolEntryId(pub(crate) u32);

impl fmt::Display for PoolEntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pool_{}", self.0)
    }
}

/// The data to be put in the data section of the asm
#[derive(Default, Clone, Debug)]
pub struct DataSection {
    pub non_configurables: Vec<Entry>,
    pub configurables: Vec<Entry>,
    pub(crate) pointer_id: FxHashMap<u64, DataId>,
    /// Indices (into `non_configurables`) of entries that have been relocated
    /// into a function's literal pool. Relocated entries are *not* removed from
    /// `non_configurables` (that would invalidate every other `DataId::idx`),
    /// but they contribute zero bytes to serialization and to offset
    /// computation, so the data section shrinks and surviving entries get
    /// compact offsets. Configurables are never relocated.
    pub(crate) relocated_non_configurables: FxHashSet<u32>,
}

impl DataSection {
    /// Get the number of entries
    pub fn num_entries(&self) -> usize {
        self.non_configurables.len() + self.configurables.len()
    }

    /// Iterate over all entries, non-configurables followed by configurables
    pub fn iter_all_entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.non_configurables
            .iter()
            .chain(self.configurables.iter())
            .cloned()
    }

    /// Iterate over the entries that are actually serialized, i.e. all entries
    /// except non-configurables that have been relocated to a literal pool.
    /// Each item is paired with its absolute (positional) index, so labels like
    /// `data_NonConfigurable_{idx}` stay stable and it is visible which indices
    /// were relocated (they are simply absent here). Used by the ASM/bytecode
    /// dumps so they reflect the real data section rather than the tombstones.
    pub(crate) fn iter_serialized_entries(&self) -> impl Iterator<Item = (usize, &Entry)> + '_ {
        self.iter_all_entries_indexed()
            .filter(|(idx, _)| !self.is_relocated_abs(*idx))
    }

    /// Iterate over all entries paired with their absolute index, non-configurables
    /// followed by configurables.
    fn iter_all_entries_indexed(&self) -> impl Iterator<Item = (usize, &Entry)> + '_ {
        self.non_configurables
            .iter()
            .chain(self.configurables.iter())
            .enumerate()
    }

    /// Returns true if the entry at absolute index `abs_idx` has been relocated
    /// to a literal pool. Only non-configurables can be relocated.
    fn is_relocated_abs(&self, abs_idx: usize) -> bool {
        abs_idx < self.non_configurables.len()
            && self.relocated_non_configurables.contains(&(abs_idx as u32))
    }

    /// Mark a non-configurable entry as relocated to a literal pool. The entry
    /// stays in `non_configurables` (preserving `DataId::idx` stability) but
    /// contributes zero bytes to the serialized data section.
    pub(crate) fn mark_relocated(&mut self, id: &DataId) {
        if id.kind == DataIdEntryKind::NonConfigurable {
            self.relocated_non_configurables.insert(id.idx);
        }
    }

    /// Un-mark a previously relocated entry, restoring it to the data section.
    ///
    /// Reserved for the Phase 2 verification/repair step ("check everything at
    /// the end"). For the current `move_to_literal_pools` pass, which only
    /// relocates *address-of* uses, the late check is the `delta <= 18-bit MOVI`
    /// assertion in `AllocatedInstruction::addr_from_literal_pool`, reached from
    /// `to_bytecode_mut` with final offsets; address-of relocation is
    /// size-neutral and bounded by the function size, so the assertion never
    /// fires in practice and no revert is needed. This helper is kept for the
    /// future value-load relocation extension, where a revert would use it.
    #[allow(dead_code)]
    pub(crate) fn unmark_relocated(&mut self, id: &DataId) {
        if id.kind == DataIdEntryKind::NonConfigurable {
            self.relocated_non_configurables.remove(&id.idx);
        }
    }

    /// Has this entry been relocated to a literal pool?
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn is_relocated(&self, id: &DataId) -> bool {
        id.kind == DataIdEntryKind::NonConfigurable
            && self.relocated_non_configurables.contains(&id.idx)
    }

    /// Get the absolute index of an id
    fn absolute_idx(&self, id: &DataId) -> usize {
        match id.kind {
            DataIdEntryKind::NonConfigurable => id.idx as usize,
            DataIdEntryKind::Configurable => id.idx as usize + self.non_configurables.len(),
        }
    }

    /// Get entry at id
    fn get(&self, id: &DataId) -> Option<&Entry> {
        match id.kind {
            DataIdEntryKind::NonConfigurable => self.non_configurables.get(id.idx as usize),
            DataIdEntryKind::Configurable => self.configurables.get(id.idx as usize),
        }
    }

    /// Given a [DataId], calculate the offset _from the beginning of the data section_ to the data
    /// in bytes.
    pub(crate) fn data_id_to_offset(&self, id: &DataId) -> usize {
        let idx = self.absolute_idx(id);
        self.absolute_idx_to_offset(idx)
    }

    /// Given an absolute index, calculate the offset _from the beginning of the data section_ to the data
    /// in bytes. Entries that have been relocated to a literal pool contribute zero
    /// bytes, so surviving entries get compact offsets while `DataId::idx` stays
    /// stable.
    pub(crate) fn absolute_idx_to_offset(&self, idx: usize) -> usize {
        self.iter_all_entries_indexed()
            .take(idx)
            .fold(0, |offset, (entry_idx, entry)| {
                if self.is_relocated_abs(entry_idx) {
                    offset
                } else {
                    //entries must be word aligned
                    size_bytes_round_up_to_word_alignment!(offset + entry.to_bytes().len())
                }
            })
    }

    pub(crate) fn serialize_to_bytes(&self) -> Vec<u8> {
        // not the exact right capacity but serves as a lower bound
        let mut buf = Vec::with_capacity(self.num_entries());
        for (entry_idx, entry) in self.iter_all_entries_indexed() {
            if self.is_relocated_abs(entry_idx) {
                continue;
            }
            buf.append(&mut entry.to_bytes());

            //entries must be word aligned
            let aligned_len = size_bytes_round_up_to_word_alignment!(buf.len());
            buf.extend(vec![0u8; aligned_len - buf.len()]);
        }
        buf
    }

    /// Returns whether a specific [DataId] value has a copy type (fits in a register).
    pub(crate) fn has_copy_type(&self, id: &DataId) -> Option<bool> {
        self.get(id).map(|entry| entry.has_copy_type())
    }

    /// Returns whether a specific [DataId] value is a byte entry.
    pub(crate) fn is_byte(&self, id: &DataId) -> Option<bool> {
        self.get(id).map(|entry| entry.is_byte())
    }

    /// When generating code, sometimes a hard-coded data pointer is needed to reference
    /// static values that have a length longer than one word.
    /// This method appends pointers to the end of the data section (thus, not altering the data
    /// offsets of previous data).
    /// `pointer_value` is in _bytes_ and refers to the offset from instruction start or
    /// relative to the current (load) instruction.
    pub(crate) fn append_pointer(&mut self, pointer_value: u64) -> DataId {
        // The 'pointer' is just a literal 64 bit address.
        let data_id = self.insert_data_value(Entry::new_word(
            pointer_value,
            EntryName::NonConfigurable,
            None,
        ));
        self.pointer_id.insert(pointer_value, data_id.clone());
        data_id
    }

    /// Get the [DataId] for a pointer, if it exists.
    /// The pointer must've been inserted with append_pointer.
    pub(crate) fn data_id_of_pointer(&self, pointer_value: u64) -> Option<DataId> {
        self.pointer_id.get(&pointer_value).cloned()
    }

    /// Serialize a function's literal pool: each entry's bytes, word-aligned, in order.
    /// This mirrors [`DataSection::serialize_to_bytes`] but for a standalone pool.
    pub(crate) fn serialize_literal_pool(entries: &[Entry]) -> Vec<u8> {
        let mut buf = Vec::new();
        for entry in entries {
            buf.append(&mut entry.to_bytes());
            let aligned_len = size_bytes_round_up_to_word_alignment!(buf.len());
            buf.extend(vec![0u8; aligned_len - buf.len()]);
        }
        buf
    }

    /// Byte offset of entry `idx` within a serialized literal pool (i.e. the
    /// distance from the start of the pool to the start of entry `idx`).
    pub(crate) fn literal_pool_entry_offset(entries: &[Entry], idx: usize) -> usize {
        entries.iter().take(idx).fold(0, |offset, entry| {
            size_bytes_round_up_to_word_alignment!(offset + entry.to_bytes().len())
        })
    }

    /// Total serialized size in bytes of a literal pool. Each entry is
    /// word-aligned, so the result is always a multiple of 8.
    pub(crate) fn literal_pool_size_bytes(entries: &[Entry]) -> u64 {
        entries.iter().fold(0u64, |offset, entry| {
            size_bytes_round_up_to_word_alignment!(offset + entry.to_bytes().len() as u64)
        })
    }

    /// Given any data in the form of a [Literal] (using this type mainly because it includes type
    /// information and debug spans), insert it into the data section and return its handle as
    /// [DataId].
    pub(crate) fn insert_data_value(&mut self, new_entry: Entry) -> DataId {
        // if there is an identical data value, use the same id

        let (value_pairs, kind) = match new_entry.name {
            EntryName::NonConfigurable => (
                &mut self.non_configurables,
                DataIdEntryKind::NonConfigurable,
            ),
            EntryName::Configurable(_) => (&mut self.configurables, DataIdEntryKind::Configurable),
        };
        match value_pairs.iter().position(|entry| entry.equiv(&new_entry)) {
            Some(num) => DataId {
                idx: num as u32,
                kind,
            },
            None => {
                value_pairs.push(new_entry);
                // the index of the data section where the value is stored
                DataId {
                    idx: (value_pairs.len() - 1) as u32,
                    kind,
                }
            }
        }
    }

    // If the stored data is Datum::Word, return the inner value.
    pub(crate) fn get_data_word(&self, data_id: &DataId) -> Option<u64> {
        let value_pairs = match data_id.kind {
            DataIdEntryKind::NonConfigurable => &self.non_configurables,
            DataIdEntryKind::Configurable => &self.configurables,
        };
        value_pairs.get(data_id.idx as usize).and_then(|entry| {
            if let Datum::Word(w) = entry.value {
                Some(w)
            } else {
                None
            }
        })
    }
}

impl fmt::Display for DataSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;
        let mut data_buf = String::new();
        // Only show entries that are actually serialized: relocated
        // non-configurables live in a function's literal pool now (their
        // `data_*` index is simply absent here), so this reflects the real
        // data section rather than the tombstones kept for `DataId` stability.
        for (ix, entry) in self.iter_serialized_entries() {
            writeln!(
                data_buf,
                "data_{}_{} {}",
                entry.name,
                ix,
                entry.display_value()
            )?;
        }

        write!(f, ".data:\n{data_buf}")
    }
}

