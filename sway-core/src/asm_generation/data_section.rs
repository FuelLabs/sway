use crate::asm_generation::from_ir::ir_type_size_in_bytes;

use sway_ir::{AggregateContent, Constant, ConstantValue, Context, Type};

use std::fmt::{self, Write};

// An entry in the data section.  It's important for the size to be correct, especially for unions
// where the size could be larger than the represented value.
#[derive(Clone, Debug)]
pub struct Entry {
    value: Datum,
    size:  usize,
}

#[derive(Clone, Debug)]
pub enum Datum {
    Word(u64),
    ByteArray(Vec<u8>),
    Collection(Vec<Entry>),
}

impl Entry {
    pub(crate) fn new_word(value: u64, size: Option<usize>) -> Entry {
        Entry {
            value: Datum::Word(value),
            size:  size.unwrap_or(8),
        }
    }

    pub(crate) fn new_byte_array(bytes: Vec<u8>, size: Option<usize>) -> Entry {
        let size = size.unwrap_or(bytes.len());
        Entry {
            value: Datum::ByteArray(bytes),
            size,
        }
    }

    pub(crate) fn new_collection(elements: Vec<Entry>, size: Option<usize>) -> Entry {
        let size = size.unwrap_or_else(|| elements.iter().map(|el| el.size).sum());
        Entry {
            value: Datum::Collection(elements),
            size,
        }
    }

    pub(crate) fn from_constant(context: &Context, constant: &Constant) -> Entry {
        // We have to do some painful special handling here for enums, which are tagged unions.
        // This really should be handled by the IR more explicitly and is something that will
        // hopefully be addressed by https://github.com/FuelLabs/sway/issues/2819#issuecomment-1256930392
        let size = Some(ir_type_size_in_bytes(context, &constant.ty) as usize);

        // Is this constant a tagged union?
        if let Type::Struct(struct_agg) = &constant.ty {
            if let AggregateContent::FieldTypes(field_tys) = struct_agg.get_content(context) {
                if field_tys.len() == 2
                    && matches!(
                        (field_tys[0], field_tys[1]),
                        (Type::Uint(_), Type::Union(_))
                    )
                {
                    // OK, this looks very much like a tagged union enum, which is the only place
                    // we use unions (otherwise we should be generalising this a bit more).
                    if let ConstantValue::Struct(els) = &constant.value {
                        if els.len() == 2 {
                            let tag_entry = Entry::from_constant(context, &els[0]);

                            // Here's the special case.  We need to get the size of the union and
                            // attach it to this constant entry which will be one of the variants.
                            let mut val_entry = Entry::from_constant(context, &els[1]);
                            val_entry.size = ir_type_size_in_bytes(context, &field_tys[1]) as usize;

                            // Return here from our special case.
                            return Entry::new_collection(vec![tag_entry, val_entry], size);
                        }
                    }
                }
            }
        };

        // Not a tagged union, no trickiness required.
        match &constant.value {
            ConstantValue::Undef | ConstantValue::Unit => Entry::new_word(0, size),
            ConstantValue::Bool(b) => Entry::new_word(if *b { 1 } else { 0 }, size),
            ConstantValue::Uint(u) => Entry::new_word(*u, size),

            ConstantValue::B256(bs) => Entry::new_byte_array(bs.to_vec(), size),
            ConstantValue::String(bs) => Entry::new_byte_array(bs.clone(), size),

            ConstantValue::Array(els) | ConstantValue::Struct(els) => Entry::new_collection(
                els.iter()
                    .map(|el| Entry::from_constant(context, el))
                    .collect(),
                size,
            ),
        }
    }

    /// Converts a literal to a big-endian representation. This is padded to words.
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        // Get the big-endian byte representation of the basic value.
        let mut bytes = match &self.value {
            Datum::Word(w) => w.to_be_bytes().to_vec(),
            Datum::ByteArray(bs) if bs.len() % 8 == 0 => bs.clone(),
            Datum::ByteArray(bs) => bs
                .iter()
                .chain(vec![0; 8].iter())
                .copied()
                .take((bs.len() + 7) & 0xfffffff8_usize)
                .collect(),
            Datum::Collection(els) => els.iter().flat_map(|el| el.to_bytes()).collect(),
        };

        // Pad the size out to match the specified size.
        if self.size > bytes.len() {
            let mut pad = vec![0; self.size - bytes.len()];
            pad.append(&mut bytes);
            bytes = pad;
        }

        bytes
    }

    pub(crate) fn has_copy_type(&self) -> bool {
        matches!(self.value, Datum::Word(_))
    }

    pub(crate) fn equiv(&self, entry: &Entry) -> bool {
        fn equiv_data(lhs: &Datum, rhs: &Datum) -> bool {
            match (lhs, rhs) {
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

        equiv_data(&self.value, &entry.value)
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
}

impl DataSection {
    /// Given a [DataId], calculate the offset _from the beginning of the data section_ to the data
    /// in bytes.
    pub(crate) fn offset_to_id(&self, id: &DataId) -> usize {
        self.value_pairs
            .iter()
            .take(id.0 as usize)
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

    /// When generating code, sometimes a hard-coded data pointer is needed to reference
    /// static values that have a length longer than one word.
    /// This method appends pointers to the end of the data section (thus, not altering the data
    /// offsets of previous data).
    /// `pointer_value` is in _bytes_ and refers to the offset from instruction start to the data
    /// in question.
    pub(crate) fn append_pointer(&mut self, pointer_value: u64) -> DataId {
        // The 'pointer' is just a literal 64 bit address.
        self.insert_data_value(Entry::new_word(pointer_value, None))
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

        write!(f, ".data:\n{}", data_buf)
    }
}
