use sway_ir::{size_bytes_round_up_to_word_alignment, Constant, ConstantValue, Context, Padding};

use std::{
    collections::BTreeMap,
    fmt::{self, Write},
    iter::repeat,
};

// An entry in the data section.  It's important for the size to be correct, especially for unions
// where the size could be larger than the represented value.
#[derive(Clone, Debug)]
pub struct Entry {
    value: Datum,
    padding: Padding,
    // It is assumed, for now, that only configuration-time constants have a name. Otherwise, this
    // is `None`.
    name: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Datum {
    Byte(u8),
    Word(u64),
    ByteArray(Vec<u8>),
    Slice(Vec<u8>),
    Collection(Vec<Entry>),
}

impl Entry {
    pub(crate) fn new_byte(value: u8, name: Option<String>, padding: Option<Padding>) -> Entry {
        Entry {
            value: Datum::Byte(value),
            padding: padding.unwrap_or(Padding::default_for_u8(value)),
            name,
        }
    }

    pub(crate) fn new_word(value: u64, name: Option<String>, padding: Option<Padding>) -> Entry {
        Entry {
            value: Datum::Word(value),
            padding: padding.unwrap_or(Padding::default_for_u64(value)),
            name,
        }
    }

    pub(crate) fn new_byte_array(
        bytes: Vec<u8>,
        name: Option<String>,
        padding: Option<Padding>,
    ) -> Entry {
        Entry {
            padding: padding.unwrap_or(Padding::default_for_byte_array(&bytes)),
            value: Datum::ByteArray(bytes),
            name,
        }
    }

    pub(crate) fn new_slice(
        bytes: Vec<u8>,
        name: Option<String>,
        padding: Option<Padding>,
    ) -> Entry {
        Entry {
            padding: padding.unwrap_or(Padding::default_for_byte_array(&bytes)),
            value: Datum::Slice(bytes),
            name,
        }
    }

    pub(crate) fn new_collection(
        elements: Vec<Entry>,
        name: Option<String>,
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
        constant: &Constant,
        name: Option<String>,
        padding: Option<Padding>,
    ) -> Entry {
        // We need a special handling in case of enums.
        if constant.ty.is_enum(context) {
            let (tag, value) = constant
                .enum_tag_and_value_with_paddings(context)
                .expect("Constant is an enum.");

            let tag_entry = Entry::from_constant(context, tag.0, None, tag.1);
            let value_entry = Entry::from_constant(context, value.0, None, value.1);

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
                    .map(|(elem, padding)| Entry::from_constant(context, elem, None, padding))
                    .collect(),
                name,
                padding,
            ),
            ConstantValue::Struct(_) => Entry::new_collection(
                constant
                    .struct_fields_with_padding(context)
                    .expect("Constant is a struct.")
                    .into_iter()
                    .map(|(elem, padding)| Entry::from_constant(context, elem, None, padding))
                    .collect(),
                name,
                padding,
            ),
            ConstantValue::RawUntypedSlice(bytes) => Entry::new_slice(bytes.clone(), name, padding),
            ConstantValue::Reference(_) => {
                todo!("Constant references are currently not supported.")
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
            Padding::Left { .. } => [repeat(0u8).take(final_padding).collect(), bytes].concat(),
            Padding::Right { .. } => [bytes, repeat(0u8).take(final_padding).collect()].concat(),
        }
    }

    pub(crate) fn has_copy_type(&self) -> bool {
        matches!(self.value, Datum::Word(_) | Datum::Byte(_))
    }

    pub(crate) fn is_byte(&self) -> bool {
        matches!(self.value, Datum::Byte(_))
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

/// An address which refers to a value in the data section of the asm.
#[derive(Clone, Debug)]
pub(crate) struct DataId(pub(crate) u32);

impl fmt::Display for DataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data_{}", self.0)
    }
}

#[derive(Default, Clone, Debug)]
pub struct DataSection {
    /// the data to be put in the data section of the asm
    pub value_pairs: Vec<Entry>,
    pub config_map: BTreeMap<String, u32>,
}

impl DataSection {
    /// Given a [DataId], calculate the offset _from the beginning of the data section_ to the data
    /// in bytes.
    pub(crate) fn data_id_to_offset(&self, id: &DataId) -> usize {
        self.raw_data_id_to_offset(id.0)
    }

    /// Given a [DataId], calculate the offset _from the beginning of the data section_ to the data
    /// in bytes.
    pub(crate) fn raw_data_id_to_offset(&self, id: u32) -> usize {
        self.value_pairs
            .iter()
            .take(id as usize)
            .fold(0, |offset, entry| {
                //entries must be word aligned
                size_bytes_round_up_to_word_alignment!(offset + entry.to_bytes().len())
            })
    }

    pub(crate) fn serialize_to_bytes(&self) -> Vec<u8> {
        // not the exact right capacity but serves as a lower bound
        let mut buf = Vec::with_capacity(self.value_pairs.len());
        for entry in &self.value_pairs {
            buf.append(&mut entry.to_bytes());

            //entries must be word aligned
            let aligned_len = size_bytes_round_up_to_word_alignment!(buf.len());
            buf.extend(vec![0u8; aligned_len - buf.len()]);
        }
        buf
    }

    /// Returns whether a specific [DataId] value has a copy type (fits in a register).
    pub(crate) fn has_copy_type(&self, id: &DataId) -> Option<bool> {
        self.value_pairs
            .get(id.0 as usize)
            .map(|entry| entry.has_copy_type())
    }

    /// Returns whether a specific [DataId] value is a byte entry.
    pub(crate) fn is_byte(&self, id: &DataId) -> Option<bool> {
        self.value_pairs
            .get(id.0 as usize)
            .map(|entry| entry.is_byte())
    }

    /// When generating code, sometimes a hard-coded data pointer is needed to reference
    /// static values that have a length longer than one word.
    /// This method appends pointers to the end of the data section (thus, not altering the data
    /// offsets of previous data).
    /// `pointer_value` is in _bytes_ and refers to the offset from instruction start or
    /// relative to the current (load) instruction.
    pub(crate) fn append_pointer(&mut self, pointer_value: u64) -> DataId {
        // The 'pointer' is just a literal 64 bit address.
        self.insert_data_value(Entry::new_word(pointer_value, None, None))
    }

    /// Given any data in the form of a [Literal] (using this type mainly because it includes type
    /// information and debug spans), insert it into the data section and return its offset as a
    /// [DataId].
    pub(crate) fn insert_data_value(&mut self, new_entry: Entry) -> DataId {
        // if there is an identical data value, use the same id
        match self
            .value_pairs
            .iter()
            .position(|entry| entry.equiv(&new_entry))
        {
            Some(num) => DataId(num as u32),
            None => {
                self.value_pairs.push(new_entry);
                // the index of the data section where the value is stored
                DataId((self.value_pairs.len() - 1) as u32)
            }
        }
    }

    // If the stored data is Datum::Word, return the inner value.
    pub(crate) fn get_data_word(&self, data_id: &DataId) -> Option<u64> {
        self.value_pairs.get(data_id.0 as usize).and_then(|entry| {
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
        fn display_entry(datum: &Datum) -> String {
            match datum {
                Datum::Byte(w) => format!(".byte {w}"),
                Datum::Word(w) => format!(".word {w}"),
                Datum::ByteArray(bs) => display_bytes_for_data_section(bs, ".bytes"),
                Datum::Slice(bs) => display_bytes_for_data_section(bs, ".slice"),
                Datum::Collection(els) => format!(
                    ".collection {{ {} }}",
                    els.iter()
                        .map(|el| display_entry(&el.value))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            }
        }

        let mut data_buf = String::new();
        for (ix, entry) in self.value_pairs.iter().enumerate() {
            writeln!(
                data_buf,
                "{} {}",
                DataId(ix as u32),
                display_entry(&entry.value)
            )?;
        }

        write!(f, ".data:\n{data_buf}")
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
