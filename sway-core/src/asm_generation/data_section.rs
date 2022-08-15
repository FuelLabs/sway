use crate::{parse_tree::*, type_system::*};
use std::fmt::{self, Write};

type Data = Literal;

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
    pub value_pairs: Vec<Data>,
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
        for val in &self.value_pairs {
            buf.append(&mut val.to_bytes().to_vec());
        }
        buf
    }

    /// Calculates the return type of the data held at a specific [DataId].
    pub(crate) fn type_of_data(&self, id: &DataId) -> Option<ResolvedType> {
        self.value_pairs.get(id.0 as usize).map(|x| x.as_type())
    }

    /// When generating code, sometimes a hard-coded data pointer is needed to reference
    /// static values that have a length longer than one word.
    /// This method appends pointers to the end of the data section (thus, not altering the data
    /// offsets of previous data).
    /// `pointer_value` is in _bytes_ and refers to the offset from instruction start to the data
    /// in question.
    pub(crate) fn append_pointer(&mut self, pointer_value: u64) -> DataId {
        let pointer_as_data = Literal::new_pointer_literal(pointer_value);
        self.insert_data_value(&pointer_as_data)
    }

    /// Given any data in the form of a [Literal] (using this type mainly because it includes type
    /// information and debug spans), insert it into the data section and return its offset as a
    /// [DataId].
    pub(crate) fn insert_data_value(&mut self, data: &Literal) -> DataId {
        // if there is an identical data value, use the same id
        match self.value_pairs.iter().position(|x| x == data) {
            Some(num) => DataId(num as u32),
            None => {
                self.value_pairs.push(data.clone());
                // the index of the data section where the value is stored
                DataId((self.value_pairs.len() - 1) as u32)
            }
        }
    }
}

impl fmt::Display for DataSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut data_buf = String::new();
        for (ix, data) in self.value_pairs.iter().enumerate() {
            let data_val = match data {
                Literal::U8(num) => format!(".u8 {:#04x}", num),
                Literal::U16(num) => format!(".u16 {:#04x}", num),
                Literal::U32(num) => format!(".u32 {:#04x}", num),
                Literal::U64(num) => format!(".u64 {:#04x}", num),
                Literal::Numeric(num) => format!(".u64 {:#04x}", num),
                Literal::Boolean(b) => format!(".bool {}", if *b { "0x01" } else { "0x00" }),
                Literal::String(st) => format!(".str \"{}\"", st.as_str()),
                Literal::Byte(b) => format!(".byte {:#08b}", b),
                Literal::B256(b) => format!(
                    ".b256 0x{}",
                    b.iter()
                        .map(|x| format!("{:02x}", x))
                        .collect::<Vec<_>>()
                        .join("")
                ),
            };
            let data_label = DataId(ix as u32);
            writeln!(data_buf, "{} {}", data_label, data_val)?;
        }

        write!(f, ".data:\n{}", data_buf)
    }
}
