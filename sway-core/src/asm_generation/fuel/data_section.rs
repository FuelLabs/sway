use rustc_hash::FxHashMap;
use sway_ir::{
    size_bytes_round_up_to_word_alignment, ConstantContent, ConstantValue, Context,
};

use std::{collections::BTreeMap, fmt};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum EntryName {
    NonConfigurable,
    Configurable(String),
}

impl fmt::Display for EntryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryName::NonConfigurable => write!(f, "NonConfigurable"),
            EntryName::Configurable(name) => write!(f, "<Configurable, {}>", name),
        }
    }
}

// An entry in the data section. It's important for the size to be correct, especially for unions
// where the size could be larger than the represented value.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Entry {
    pub value: Datum,
    pub name: EntryName,
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum Datum {
    /// A single byte, loaded into a register.
    U8(u8),
    /// A quarterword, loaded into a register.
    U16(u16),
    /// A halfword, loaded into a register.
    U32(u32),
    /// A word, loaded into a register.
    U64(u64),
    /// Data behind a pointer.
    ByRef(Vec<u8>),
    /// Collection of entries.
    Collection(Vec<Entry>),
}

impl Entry {
    pub(crate) fn new_byte(value: u8, name: EntryName) -> Entry {
        Entry {
            value: Datum::U8(value),
            name,
        }
    }

    pub(crate) fn new_word(value: u64, name: EntryName) -> Entry {
        Entry {
            value: Datum::U64(value),
            name,
        }
    }

    pub(crate) fn new_byte_array(
        bytes: Vec<u8>,
        name: EntryName,
    ) -> Entry {
        Entry {
            value: Datum::ByRef(bytes),
            name,
        }
    }

    pub(crate) fn new_collection(
        elements: Vec<Entry>,
        name: EntryName,
    ) -> Entry {
        Entry {
            value: Datum::Collection(elements),
            name,
        }
    }

    pub(crate) fn from_constant(
        context: &Context,
        constant: &ConstantContent,
        name: EntryName,
    ) -> Entry {
        // We need a special handling in case of enums.
        if constant.ty.is_enum(context) {
            let elems = match &constant.value {
                ConstantValue::Struct(elems) if elems.len() == 2 => elems,
                _ => unreachable!("enums are represented as structs with 2 elements"),
            };
            let tag = &elems[0];
            let value = &elems[1];

            let tag_entry = Entry::from_constant(context, tag, EntryName::NonConfigurable);
            let value_entry =
                Entry::from_constant(context, value, EntryName::NonConfigurable);

            return Entry::new_collection(vec![tag_entry, value_entry], name);
        }

        // Not an enum, no more special handling required.
        match &constant.value {
            ConstantValue::Undef | ConstantValue::Unit => Entry::new_byte(0, name),
            ConstantValue::Bool(value) => Entry::new_byte(u8::from(*value), name),
            ConstantValue::Uint(value) => {
                if constant.ty.is_uint8(context) {
                    Entry {
                        value: Datum::U8(*value as u8),
                        name,
                    }
                } else if constant.ty.is_uint16(context) {
                    Entry {
                        value: Datum::U16(*value as u16),
                        name,
                    }
                } else if constant.ty.is_uint32(context) {
                    Entry {
                        value: Datum::U32(*value as u32),
                        name,
                    }
                } else {
                    Entry {
                        value: Datum::U64(*value as u64),
                        name,
                    }
                }
            }
            ConstantValue::U256(value) => {
                Entry::new_byte_array(value.to_be_bytes().to_vec(), name)
            }
            ConstantValue::B256(value) => {
                Entry::new_byte_array(value.to_be_bytes().to_vec(), name)
            }
            ConstantValue::String(bytes) => Entry::new_byte_array(bytes.clone(), name),
            ConstantValue::Array(elements) => Entry::new_collection(
                elements
                    .into_iter()
                    .map(|elem| {
                        Entry::from_constant(context, elem, EntryName::NonConfigurable)
                    })
                    .collect(),
                name,
            ),
            ConstantValue::Struct(fields) => Entry::new_collection(
                fields
                    .into_iter()
                    .map(|elem| {
                        Entry::from_constant(context, elem, EntryName::NonConfigurable)
                    })
                    .collect(),
                name,
            ),
            ConstantValue::RawUntypedSlice(bytes) => Entry::new_byte_array(bytes.clone(), name),
            ConstantValue::Reference(_) => {
                todo!("Constant references are currently not supported.")
            }
            ConstantValue::Slice(_) => {
                todo!("Constant slices are currently not supported.")
            }
        }
    }

    /// Converts a literal to a big-endian representation. No padding is applied.
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        match &self.value {
            Datum::U8(v) => v.to_be_bytes().to_vec(),
            Datum::U16(v) => v.to_be_bytes().to_vec(),
            Datum::U32(v) => v.to_be_bytes().to_vec(),
            Datum::U64(v) => v.to_be_bytes().to_vec(),
            Datum::ByRef(bytes) => bytes.clone(),
            Datum::Collection(items) => items.iter().flat_map(|el| el.to_bytes()).collect(),
        }
    }

    pub(crate) fn has_copy_type(&self) -> bool {
        matches!(self.value, Datum::U8(_) | Datum::U16(_) | Datum::U32(_) | Datum::U64(_))
    }

    /// Size of the entry in bytes.
    pub(crate) fn size(&self) -> usize {
        match &self.value {
            Datum::U8(_) => 1,
            Datum::U16(_) => 2,
            Datum::U32(_) => 4,
            Datum::U64(_) => 8,
            Datum::ByRef(bytes) => bytes.len(),
            Datum::Collection(items) => items.iter().map(|el| el.size()).sum(),
        }
    }

    pub(crate) fn equiv(&self, entry: &Entry) -> bool {
        fn equiv_data(lhs: &Datum, rhs: &Datum) -> bool {
            match (lhs, rhs) {
                (Datum::U8(l), Datum::U8(r)) => l == r,
                (Datum::U16(l), Datum::U16(r)) => l == r,
                (Datum::U32(l), Datum::U32(r)) => l == r,
                (Datum::U64(l), Datum::U64(r)) => l == r,
                (Datum::ByRef(l), Datum::ByRef(r)) => l == r,
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

#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
pub(crate) struct DataId {
    pub(crate) idx: u32,
    pub(crate) kind: DataIdEntryKind,
}

impl fmt::Display for DataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data_{}_{}", self.kind, self.idx)
    }
}

/// The data to be put in the data section of the asm.
/// After all data has been inserted, an immutable [FinalDataSection] is created.
#[derive(Default, Clone, Debug)]
pub struct DataSection {
    pub non_configurables: Vec<Entry>,
    pub configurables: Vec<Entry>,
    pub(crate) pointer_id: FxHashMap<u64, DataId>,
}

impl DataSection {
    /// Get entry at id
    fn get(&self, id: &DataId) -> Option<&Entry> {
        match id.kind {
            DataIdEntryKind::NonConfigurable => self.non_configurables.get(id.idx as usize),
            DataIdEntryKind::Configurable => self.configurables.get(id.idx as usize),
        }
    }

    /// Returns whether a specific [DataId] value has a copy type (fits in a register).
    pub(crate) fn has_copy_type(&self, id: &DataId) -> Option<bool> {
        self.get(id).map(|entry| entry.has_copy_type())
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

    /// When a load from data section is realized and targets a (register-placeable) copy type,
    /// this is the value that will be loaded into the register.
    /// For non-copy types, returns `None` instead.
    pub(crate) fn get_reg_value(&self, data_id: DataId) -> Option<u64> {
        let entry = self.get(&data_id)?;
        match &entry.value {
            Datum::U8(v) => Some(*v as u64),
            Datum::U16(v) => Some(*v as u64),
            Datum::U32(v) => Some(*v as u64),
            Datum::U64(v) => Some(*v),
            _ => None,
        }
    }
    
    pub(crate) fn pack(&self, optimization: crate::OptLevel) -> PackedDataSection {
        PackedDataSection::new(self, optimization)
    }
}

fn display_bytes_for_data_section(bs: &Vec<u8>, prefix: &str) -> String {
    let mut hex_str = String::new();
    let mut chr_str = String::new();
    for b in bs {
        hex_str.push_str(format!("{b:02x} ").as_str());
        chr_str.push(if *b == b' ' || b.is_ascii_graphic() {
            *b as char
        } else {
            '.'
        });
    }
    format!("{prefix}[{}] {hex_str} {chr_str}", bs.len())
}


/// Data section packed into it's penultimate form,
/// ready for label serialization. Still allows pointer insertion,
/// but no other modifications.
#[derive(Default, Clone, Debug)]
pub struct PackedDataSection {
    non_configurables: Vec<Entry>,
    configurables: Vec<Entry>,
    pointer_id: FxHashMap<u64, DataId>,
}

impl PackedDataSection {
    pub(crate) fn new(
        data_section: &DataSection,
        _optimization: crate::OptLevel,
    ) -> PackedDataSection {
        // TODO: optimize
        
        let mut packed = PackedDataSection::default();
        packed.non_configurables = data_section
            .non_configurables
            .iter()
            .map(|entry| entry.clone())
            .collect();
        packed.configurables = data_section
            .configurables
            .iter()
            .map(|entry| entry.clone())
            .collect();
        packed.pointer_id = data_section.pointer_id.clone();
        packed
    }

    pub(crate) fn finalize(self) -> FinalDataSection {
        FinalDataSection { non_configurables: self.non_configurables, configurables: self.configurables }
    }

    /// When generating code, sometimes a hard-coded data pointer is needed to reference
    /// static values that have a length longer than one word.
    /// This method appends pointers to the end of the data section (thus, not altering the data
    /// offsets of previous data).
    /// `pointer_value` is in _bytes_ and refers to the offset from instruction start or
    /// relative to the current (load) instruction.
    pub(crate) fn append_pointer(&mut self, pointer_value: u64) -> DataId {
        todo!();
        // // The 'pointer' is just a literal 64 bit address.
        // let data_id = self.insert_data_value(Entry::new_word(
        //     pointer_value,
        //     EntryName::NonConfigurable,
        // ));
        // self.pointer_id.insert(pointer_value, data_id.clone());
        // data_id
    }

    /// Get the [DataId] for a pointer, if it exists.
    /// The pointer must've been inserted with append_pointer.
    pub(crate) fn data_id_of_pointer(&self, pointer_value: u64) -> Option<DataId> {
        self.pointer_id.get(&pointer_value).cloned()
    }
    
    /// Return offsets to named configurables.
    pub(crate) fn named_offsets(&self, offset_to_data_section_in_bytes: u64) -> BTreeMap<String, u64> {
        // self.configurables.iter().enumerate()
        todo!();
    }
}

/// Data section with all data laid out.
#[derive(Default, Clone, Debug)]
pub struct FinalDataSection {
    pub non_configurables: Vec<Entry>,
    pub configurables: Vec<Entry>,
}

impl FinalDataSection {
    /// Iterate over all entries, non-configurables followed by configurables
    pub fn iter_all_entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.non_configurables
            .iter()
            .chain(self.configurables.iter())
            .cloned()
    }
}

impl fmt::Display for FinalDataSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn display_entry(datum: &Datum) -> String {
            match datum {
                Datum::U8(v) => format!(".byte {v}"),
                Datum::U16(v) => format!(".quarterword {v}"),
                Datum::U32(v) => format!(".word {v}"),
                Datum::U64(v) => format!(".half {v}"),
                Datum::ByRef(bs) => display_bytes_for_data_section(bs, ".bytes"),
                Datum::Collection(els) => format!(
                    ".collection {{ {} }}",
                    els.iter()
                        .map(|el| display_entry(&el.value))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }
        }

        use std::fmt::Write;
        let mut data_buf = String::new();
        for (ix, entry) in self.iter_all_entries().enumerate() {
            writeln!(
                data_buf,
                "data_{}_{} {}",
                entry.name,
                ix,
                display_entry(&entry.value)
            )?;
        }

        write!(f, ".data:\n{data_buf}")
    }
}
