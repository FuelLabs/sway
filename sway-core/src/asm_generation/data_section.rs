use crate::asm_generation::from_ir::ir_type_size_in_bytes;

use sway_ir::{AggregateContent, Constant, ConstantValue, Context, Type};

use std::fmt::{self, Write};

/// An address which refers to a value in the data section of the asm.
#[derive(Clone, Debug)]
pub(crate) struct DataId(usize);

impl fmt::Display for DataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data_{}", self.0)
    }
}

// An entry in the data section.  It's important for the size to be correct, especially for unions
// where the size could be larger than the represented value.
#[derive(Clone, Debug)]
pub struct Entry {
    value: Datum,
    size: usize,
}

#[derive(Clone, Debug)]
pub(crate) enum Datum {
    Word(u64),
    ByteArray(Vec<u8>),
    Collection(Vec<Entry>),
    Reference(DataId),
    Offset(u64),
}

impl Entry {
    pub(crate) fn new_word(value: u64, size: Option<usize>) -> Entry {
        Entry {
            value: Datum::Word(value),
            size: size.unwrap_or(8),
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

    pub(crate) fn new_ref(data_id: DataId) -> Entry {
        Entry {
            value: Datum::Reference(data_id),
            size: 8,
        }
    }

    pub(crate) fn new_offset(offset: u64) -> Entry {
        Entry {
            value: Datum::Offset(offset),
            size: 8,
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
            ConstantValue::Bool(b) => Entry::new_word(u64::from(*b), size),
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
            Datum::Word(w) | Datum::Offset(w) => w.to_be_bytes().to_vec(),
            Datum::ByteArray(bs) if bs.len() % 8 == 0 => bs.clone(),
            Datum::ByteArray(bs) => bs
                .iter()
                .chain(vec![0; 8].iter())
                .copied()
                .take((bs.len() + 7) & 0xfffffff8_usize)
                .collect(),
            Datum::Collection(els) => els.iter().flat_map(|el| el.to_bytes()).collect(),

            Datum::Reference(_) => unreachable!(
                "We will move this to the RealizedSections which doesn't have symbolic references."
            ),
        };

        // Pad the size out to match the specified size.
        if self.size > bytes.len() {
            let mut pad = vec![0; self.size - bytes.len()];
            pad.append(&mut bytes);
            bytes = pad;
        }

        bytes
    }

    fn byte_len(&self) -> usize {
        std::cmp::max(
            self.size,
            match &self.value {
                Datum::Word(_) | Datum::Reference(_) | Datum::Offset(_) => 8,
                Datum::ByteArray(bs) => (bs.len() + 7) & 0xfffffff8_usize,
                Datum::Collection(els) => els.iter().map(|el| el.byte_len()).sum(),
            },
        )
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

#[derive(Default, Clone, Debug)]
pub struct VirtualDataSection {
    /// the data to be put in the data section of the asm
    pub value_pairs: Vec<Entry>,
}

impl VirtualDataSection {
    /// Given a [DataId], calculate the offset _from the beginning of the data section_ to the data
    /// in bytes.
    pub(crate) fn offset_to_id(&self, id: &DataId) -> usize {
        self.value_pairs
            .iter()
            .take(id.0 as usize)
            .map(|entry| entry.byte_len())
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

    pub(crate) fn finalize(mut self, data_section_base_offset: u64) -> VirtualDataSection {
        // Create a new data section with references replaced with words containing the final data
        // offset to the referee.  The references are always to the previous entry, so we just need
        // to track the most recent offset.

        self.value_pairs.iter_mut().enumerate().fold(
            (0, 0),
            |(prev_offs, prev_size), (cur_idx, entry)| {
                let cur_offs = prev_offs + prev_size;
                let cur_size = entry.byte_len();

                if let Datum::Reference(DataId(prev_idx)) = entry.value {
                    assert!(prev_idx == cur_idx - 1, "{prev_idx} != {cur_idx} - 1");
                    *entry = Entry::new_offset(prev_offs as u64 + data_section_base_offset);
                }

                (cur_offs, cur_size)
            },
        );

        self
    }

    pub(crate) fn is_reference(&self, data_id: &DataId) -> bool {
        matches!(&self.value_pairs[data_id.0].value, Datum::Reference(_))
    }

    pub(crate) fn is_offset(&self, data_id: &DataId) -> bool {
        matches!(&self.value_pairs[data_id.0].value, Datum::Offset(_))
    }

    /// Insert an Entry into the data section, return a [DataId] to it.
    pub(crate) fn insert_data_value(&mut self, new_entry: Entry) -> DataId {
        let is_large_entry = matches!(&new_entry.value, Datum::ByteArray(_) | Datum::Collection(_));

        // If there is an identical data value use the same id.
        match self
            .value_pairs
            .iter()
            .position(|entry| entry.equiv(&new_entry))
        {
            None => {
                // The index of the data section where the value is stored.
                self.value_pairs.push(new_entry);
                let data_id = DataId(self.value_pairs.len() - 1);

                // If the entry is a too large to fit in a word then we insert a reference to it, and
                // return the reference.
                if is_large_entry {
                    self.insert_data_value(Entry::new_ref(data_id))
                } else {
                    data_id
                }
            }

            Some(num) => {
                // Minor hackiness: if it's a large entry then we want its reference, which is the
                // following entry.
                if is_large_entry {
                    assert!(matches!(
                        self.value_pairs[num + 1].value,
                        Datum::Reference(DataId(_))
                    ));
                    DataId(num + 1)
                } else {
                    DataId(num)
                }
            }
        }
    }
}

impl fmt::Display for VirtualDataSection {
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
                Datum::Reference(data_id) => format!(".ref {data_id}"),
                Datum::Offset(offs) => format!(".offset {offs}"),
            }
        }

        let mut data_buf = String::new();
        for (ix, entry) in self.value_pairs.iter().enumerate() {
            writeln!(data_buf, "{} {}", DataId(ix), display_entry(&entry.value))?;
        }

        write!(f, ".data:\n{}", data_buf)
    }
}
