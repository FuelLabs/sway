use crate::{
    asm_generation::from_ir::ir_type_size_in_bytes, size_bytes_round_up_to_word_alignment,
};

use sway_ir::{Constant, ConstantValue, Context};

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
    Collection(Vec<Entry>),
}

#[derive(Clone, Debug)]
pub(crate) enum Padding {
    Left { target_size: usize },
    Right { target_size: usize },
}

impl Padding {
    pub fn target_size(&self) -> usize {
        use Padding::*;
        match self {
            Left { target_size } | Right { target_size } => *target_size,
        }
    }
}

impl Entry {
    pub(crate) fn new_byte(value: u8, name: Option<String>, padding: Option<Padding>) -> Entry {
        Entry {
            value: Datum::Byte(value),
            padding: padding.unwrap_or(Padding::Right { target_size: 1 }),
            name,
        }
    }

    pub(crate) fn new_word(value: u64, name: Option<String>, padding: Option<Padding>) -> Entry {
        Entry {
            value: Datum::Word(value),
            padding: padding.unwrap_or(Padding::Right { target_size: 8 }),
            name,
        }
    }

    pub(crate) fn new_byte_array(
        bytes: Vec<u8>,
        name: Option<String>,
        padding: Option<Padding>,
    ) -> Entry {
        Entry {
            padding: padding.unwrap_or(Padding::Right {
                target_size: bytes.len(),
            }),
            value: Datum::ByteArray(bytes),
            name,
        }
    }

    pub(crate) fn new_collection(
        elements: Vec<Entry>,
        name: Option<String>,
        padding: Option<Padding>,
    ) -> Entry {
        Entry {
            padding: padding.unwrap_or(Padding::Right {
                target_size: elements.iter().map(|el| el.padding.target_size()).sum(),
            }),
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
        // We have to do some painful special handling here for enums, which are tagged unions.
        // This really should be handled by the IR more explicitly and is something that will
        // hopefully be addressed by https://github.com/FuelLabs/sway/issues/2819#issuecomment-1256930392

        // Is this constant a tagged union?
        if constant.ty.is_struct(context) {
            let field_tys = constant.ty.get_field_types(context);
            if field_tys.len() == 2
                && field_tys[0].is_uint(context)
                && field_tys[1].is_union(context)
            {
                // OK, this looks very much like a tagged union enum, which is the only place
                // we use unions (otherwise we should be generalising this a bit more).
                if let ConstantValue::Struct(els) = &constant.value {
                    if els.len() == 2 {
                        let tag_entry = Entry::from_constant(context, &els[0], None, None);

                        // Here's the special case.  We need to get the size of the union and
                        // attach it to this constant entry which will be one of the variants.
                        let val_entry = {
                            let target_size = size_bytes_round_up_to_word_alignment!(
                                ir_type_size_in_bytes(context, &field_tys[1]) as usize
                            );
                            Entry::from_constant(
                                context,
                                &els[1],
                                None,
                                Some(Padding::Left { target_size }),
                            )
                        };

                        // Return here from our special case.
                        return Entry::new_collection(vec![tag_entry, val_entry], name, padding);
                    }
                }
            }
        };

        // Not a tagged union, no trickiness required.
        match &constant.value {
            ConstantValue::Undef | ConstantValue::Unit => Entry::new_byte(0, name, padding),
            ConstantValue::Bool(b) => Entry::new_byte(u8::from(*b), name, padding),
            ConstantValue::Uint(u) => {
                if constant.ty.is_uint8(context) {
                    Entry::new_byte(*u as u8, name, padding)
                } else {
                    Entry::new_word(*u, name, padding)
                }
            }
            ConstantValue::U256(u) => {
                Entry::new_byte_array(u.to_be_bytes().to_vec(), name, padding)
            }
            ConstantValue::B256(bs) => {
                Entry::new_byte_array(bs.to_be_bytes().to_vec(), name, padding)
            }
            ConstantValue::String(bs) => Entry::new_byte_array(bs.clone(), name, padding),

            ConstantValue::Array(els) => Entry::new_collection(
                els.iter()
                    .map(|el| Entry::from_constant(context, el, None, None))
                    .collect(),
                name,
                padding,
            ),
            ConstantValue::Struct(els) => Entry::new_collection(
                els.iter()
                    .map(|el| {
                        let target_size = size_bytes_round_up_to_word_alignment!(
                            ir_type_size_in_bytes(context, &el.ty) as usize
                        );
                        Entry::from_constant(
                            context,
                            el,
                            None,
                            Some(Padding::Right { target_size }),
                        )
                    })
                    .collect(),
                name,
                padding,
            ),
        }
    }

    /// Converts a literal to a big-endian representation. This is padded to words.
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        // Get the big-endian byte representation of the basic value.
        let bytes = match &self.value {
            Datum::Byte(b) => vec![*b],
            Datum::Word(w) => w.to_be_bytes().to_vec(),
            Datum::ByteArray(bs) if bs.len() % 8 == 0 => bs.clone(),
            Datum::ByteArray(bs) => bs
                .iter()
                .chain([0; 8].iter())
                .copied()
                .take((bs.len() + 7) & 0xfffffff8_usize)
                .collect(),
            Datum::Collection(els) => els.iter().flat_map(|el| el.to_bytes()).collect(),
        };

        match self.padding {
            Padding::Left { target_size } => {
                let target_size = size_bytes_round_up_to_word_alignment!(target_size);
                let left_pad = target_size.saturating_sub(bytes.len());
                [repeat(0u8).take(left_pad).collect(), bytes].concat()
            }
            Padding::Right { target_size } => {
                let target_size = size_bytes_round_up_to_word_alignment!(target_size);
                let right_pad = target_size.saturating_sub(bytes.len());
                [bytes, repeat(0u8).take(right_pad).collect()].concat()
            }
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
            .map(|x| x.to_bytes().len())
            .sum()
    }

    pub(crate) fn serialize_to_bytes(&self) -> Vec<u8> {
        // not the exact right capacity but serves as a lower bound
        let mut buf = Vec::with_capacity(self.value_pairs.len());
        for entry in &self.value_pairs {
            buf.append(&mut entry.to_bytes());
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
    /// `pointer_value` is in _bytes_ and refers to the offset from instruction start to the data
    /// in question.
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
}

impl fmt::Display for DataSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn display_entry(datum: &Datum) -> String {
            match datum {
                Datum::Byte(w) => format!(".byte {w}"),
                Datum::Word(w) => format!(".word {w}"),
                Datum::ByteArray(bs) => {
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
                    format!(".bytes[{}] {hex_str} {chr_str}", bs.len())
                }
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
