use rustc_hash::FxHashMap;
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

/// An entry in the [DataSection]. It's important for the size to be correct, especially for unions
/// where the size could be larger than the represented value.
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

    pub(crate) fn equiv(&self, entry: &Entry) -> bool {
        fn equiv_data(lhs: &Datum, rhs: &Datum) -> bool {
            match (lhs, rhs) {
                (Datum::Byte(l), Datum::Byte(r)) => l == r,
                (Datum::Word(l), Datum::Word(r)) => l == r,
                (Datum::ByteArray(l), Datum::ByteArray(r)) => l == r,
                (Datum::Slice(l), Datum::Slice(r)) => l == r,
                (Datum::Collection(l), Datum::Collection(r)) => {
                    l.len() == r.len()
                        && l.iter()
                            .zip(r.iter())
                            .all(|(l, r)| equiv_data(&l.value, &r.value))
                }
                _ => false,
            }
        }

        // If this corresponds to a configurable, then the entry names will be
        // available and they must be the same before we can merge the two
        // entries. Otherwise, `self.name` and `entry.name` will be `None`
        // in which case we're also allowed to merge the two entries
        // (if their values are equivalent of course).
        equiv_data(&self.value, &entry.value) && self.name == entry.name
    }
}

/// [DataSection] consists of three distinguished regions, laid in the
/// fixed order:
/// - non-configurables (compile-time constants)
/// - data section pointers, inserted during ASM compilation
/// - configurables
#[derive(Clone, Debug)]
pub enum DataSectionRegion {
    /// Contains compile-time constants.
    NonConfigurables,
    /// Contains hard-coded pointers (addresses) to other entries in the data section.
    ///
    /// The size of this section is fixed/reserved during the final program generation
    /// and the actual pointers are inserted during final program finalization
    /// (see [DataSection::reserve_pointer_slots] and [DataSection::append_pointer]).
    Pointers,
    /// Contains configurables.
    Configurables,
}

impl fmt::Display for DataSectionRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // TODO: Intentionally keep display string in singular to keep
            //       it compatible with the display of `Entry::name`.
            //       We want to improve displaying of data section entries
            //       in general to improve readability and troubleshooting.
            //       Until then, keeping singular here will be sufficient.
            DataSectionRegion::NonConfigurables => write!(f, "NonConfigurable"),
            DataSectionRegion::Pointers => write!(f, "Pointer"),
            DataSectionRegion::Configurables => write!(f, "Configurable"),
        }
    }
}

/// An address which refers to a value in the [DataSection].
#[derive(Clone, Debug)]
pub(crate) struct DataId {
    pub(crate) idx: u32,
    pub(crate) region: DataSectionRegion,
}

impl fmt::Display for DataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data_{}_{}", self.region, self.idx)
    }
}

/// The data section of the ASM.
#[derive(Default, Clone, Debug)]
pub struct DataSection {
    pub non_configurables: Vec<Entry>,
    pub configurables: Vec<Entry>,
    /// Hard-coded pointers (addresses) to other entries in the data section.
    ///
    /// The pointer slots are reserved upfront (see [Self::reserve_pointer_slots]), one
    /// slot per non-copy load, before jump labels are resolved, and are only filled
    /// in-place later on (see [Self::append_pointer]). This way the layout of the data
    /// section, and with it the sizes of all the instructions, are fixed before any
    /// instruction offsets or pointer values are calculated.
    pub(crate) pointers: Vec<Entry>,
    pub(crate) pointer_id: FxHashMap<u64, DataId>,
    /// The worst-case (largest possible) offset, in bytes, at which the configurables
    /// region can start, frozen before jump labels are resolved.
    /// See [Self::freeze_configurables_base_offset].
    frozen_configurables_base_offset: Option<u64>,
    /// The precomputed offset, in bytes, of every entry (by its absolute index),
    /// built once the layout of the data section is final.
    /// See [Self::freeze_layout].
    frozen_offsets: Option<Vec<usize>>,
}

impl DataSection {
    /// Get the number of entries
    pub fn num_entries(&self) -> usize {
        self.non_configurables.len() + self.pointers.len() + self.configurables.len()
    }

    /// Iterate over the all entries in the order of regions:
    /// non-configurables, then pointers, then configurables.
    pub fn iter_all_entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.non_configurables
            .iter()
            .chain(self.pointers.iter())
            .chain(self.configurables.iter())
            .cloned()
    }

    /// Get the absolute index of an id
    fn absolute_idx(&self, id: &DataId) -> usize {
        match id.region {
            DataSectionRegion::NonConfigurables => id.idx as usize,
            DataSectionRegion::Pointers => id.idx as usize + self.non_configurables.len(),
            DataSectionRegion::Configurables => {
                id.idx as usize + self.non_configurables.len() + self.pointers.len()
            }
        }
    }

    /// Get entry at id
    fn get(&self, id: &DataId) -> Option<&Entry> {
        match id.region {
            DataSectionRegion::NonConfigurables => self.non_configurables.get(id.idx as usize),
            DataSectionRegion::Configurables => self.configurables.get(id.idx as usize),
            DataSectionRegion::Pointers => self.pointers.get(id.idx as usize),
        }
    }

    /// Given a [DataId], calculate the offset _from the beginning of the data section_ to the data
    /// in bytes.
    pub(crate) fn data_id_to_offset(&self, id: &DataId) -> usize {
        let idx = self.absolute_idx(id);
        self.absolute_idx_to_offset(idx)
    }

    /// Given an absolute index, calculate the offset _from the beginning of the data section_ to the data
    /// in bytes.
    pub(crate) fn absolute_idx_to_offset(&self, idx: usize) -> usize {
        if let Some(offsets) = &self.frozen_offsets {
            offsets[idx]
        } else {
            self.iter_all_entries().take(idx).fold(0, |offset, entry| {
                // Entries must be word aligned.
                size_bytes_round_up_to_word_alignment!(offset + entry.to_bytes().len())
            })
        }
    }

    /// Freezes the layout of the data section and precomputes the offset of every
    /// entry, turning every later offset calculation into a simple lookup.
    ///
    /// Must be called only once the layout is truly final, i.e., after the jumps are
    /// realized (realizing a far jump inserts its target word into the data section).
    ///
    /// Inserting entries after the freeze is a hard error (see [Self::insert_data_value]).
    ///
    /// Note that filling the reserved pointer slots (see [Self::append_pointer]) is
    /// still possible and expected. It changes the values, but not the layout.
    ///
    /// Panics if the layout was already frozen.
    pub(crate) fn freeze_layout(&mut self) {
        assert!(
            self.frozen_offsets.is_none(),
            "the layout of the data section must be frozen exactly once",
        );
        let mut offsets = Vec::with_capacity(self.num_entries());
        let mut offset = 0;
        for entry in self.iter_all_entries() {
            offsets.push(offset);
            // Entries must be word aligned.
            offset = size_bytes_round_up_to_word_alignment!(offset + entry.to_bytes().len());
        }
        self.frozen_offsets = Some(offsets);
    }

    pub(crate) fn serialize_to_bytes(&self) -> Vec<u8> {
        // Not the exact right capacity but serves as a lower bound.
        let mut buf = Vec::with_capacity(self.num_entries());
        for entry in self.iter_all_entries() {
            buf.append(&mut entry.to_bytes());

            // Entries must be word aligned.
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
    ///
    /// `pointer_value` is in _bytes_ and refers to the **offset relative to the current (load) instruction**.
    pub(crate) fn append_pointer(&mut self, pointer_value: u64) -> DataId {
        // Pointers are deduplicated by value and fill the pointer slots reserved by
        // `reserve_pointer_slots`. Filling a slot in-place never changes the layout of
        // the data section, so all the entry offsets, and with them the instruction
        // sizes, stay exactly as they were when the jump labels were resolved.
        if let Some(data_id) = self.pointer_id.get(&pointer_value) {
            return data_id.clone();
        }
        // Due to the deduplication, the number of the filled slots so far can be smaller
        // than the number of the reserved slots, but never larger, since exactly one slot
        // was reserved for each non-copy load. Any remaining unfilled slots stay zeroed.
        let slot_idx = self.pointer_id.len();
        assert!(
            slot_idx < self.pointers.len(),
            "all reserved data section pointer slots are already filled",
        );
        // The 'pointer' is just a literal 64 bit address.
        self.pointers[slot_idx] = Entry::new_word(pointer_value, EntryName::NonConfigurable, None);
        let data_id = DataId {
            idx: slot_idx as u32,
            region: DataSectionRegion::Pointers,
        };
        self.pointer_id.insert(pointer_value, data_id.clone());
        data_id
    }

    /// Reserves `count` zeroed pointer slots, one for each non-copy
    /// `AllocatedInstruction::LoadDataId` in the program.
    ///
    /// This method **must be called before jump labels are resolved**,
    /// so that the layout of the data section is fixed before any
    /// instruction offsets are calculated.
    ///
    /// The slots are later filled in-place by [Self::append_pointer].
    ///
    /// The exact pointer values are not known at this point and it can
    /// be that some of the values will be duplicated. [Self::append_pointer]
    /// deduplicates them later on, leaving some of the reserved pointer slots unused.
    ///
    /// Making the number of slots dependent on the values would make the
    /// layout dependent on the offsets and vice versa, which is exactly what
    /// we are avoiding with reserving the fixed number of slots and freezing the layout.
    ///
    /// The price are occasional unused, zeroed slots, 8 bytes each,
    /// in the rare case when two loads share the pointer value.
    /// Measurements of real-life programs show that this situation is indeed
    /// very rare and if occurring at all, increases the data section for just
    /// a handful <5 number of 8 bytes slots.
    pub(crate) fn reserve_pointer_slots(&mut self, count: usize) {
        assert!(
            self.pointers.is_empty() && self.pointer_id.is_empty(),
            "pointer slots must be reserved exactly once, before any pointers are appended",
        );
        self.pointers = vec![Entry::new_word(0, EntryName::NonConfigurable, None); count];
    }

    /// Freezes the worst-case (largest possible) offset at which the configurables region
    /// can start. `worst_case_late_insertions_in_bytes` is the total size of the
    /// non-configurable entries that can still be inserted after this point (e.g., the
    /// far jump target words inserted while resolved jumps are being realized).
    ///
    /// Once frozen, the decision whether an `AddrDataId` pointing to a configurable
    /// needs one or two instructions (see [Self::addr_is_far]) is derived from this
    /// worst-case offset and never changes again, even if the actual configurables
    /// offset ends up smaller. This keeps all the instruction sizes stable from the
    /// moment the jump labels are resolved until the bytecode is emitted.
    pub(crate) fn freeze_configurables_base_offset(
        &mut self,
        worst_case_late_insertions_in_bytes: u64,
    ) {
        assert!(
            self.frozen_configurables_base_offset.is_none(),
            "configurables base offset must be frozen exactly once",
        );
        let configurables_start =
            self.absolute_idx_to_offset(self.non_configurables.len() + self.pointers.len()) as u64;
        self.frozen_configurables_base_offset =
            Some(configurables_start + worst_case_late_insertions_in_bytes);
    }

    /// Returns true if the `AddrDataId` instruction for the given [DataId] must be realized
    /// into the two-instruction far form (`MOVI` + `ADD`), and false if the
    /// one-instruction near form (`ADDI`) suffices.
    ///
    /// For configurables the decision is made against the frozen worst-case offset of
    /// the configurables region (see [Self::freeze_configurables_base_offset]), because
    /// their actual offset can still decrease while jumps are being realized. Since the
    /// actual offset can never exceed the worst-case one, a near decision always stays
    /// realizable, and the far form is realizable for any offset. For all other entries
    /// the actual offset is already final and is used directly.
    ///
    /// This decision must be perfectly stable: it defines the size of the instruction,
    /// and all the sizes must remain exactly the same from the moment the jump labels
    /// are resolved until the bytecode is emitted.
    ///
    /// Strictly seeing, this can lead to a pessimistic decision of using a far decision
    /// where a near one could be sufficient. In practice though, as the measurements confirm,
    /// this will almost never be the case.
    pub(crate) fn addr_is_far(&self, id: &DataId) -> bool {
        let sizing_offset = match id.region {
            DataSectionRegion::Configurables => {
                let base = self.frozen_configurables_base_offset.expect(
                    "configurables base offset must be frozen before sizing `AddrDataId` instructions",
                );
                let offset_within_configurables = self
                    .configurables
                    .iter()
                    .take(id.idx as usize)
                    .fold(0, |offset, entry| {
                        size_bytes_round_up_to_word_alignment!(offset + entry.to_bytes().len())
                    }) as u64;
                base + offset_within_configurables
            }
            DataSectionRegion::NonConfigurables | DataSectionRegion::Pointers => {
                self.data_id_to_offset(id) as u64
            }
        };
        sizing_offset > crate::asm_generation::fuel::compiler_constants::TWELVE_BITS
    }

    /// Get the [DataId] for a pointer, if it exists.
    /// The pointer must've been inserted with append_pointer.
    pub(crate) fn data_id_of_pointer(&self, pointer_value: u64) -> Option<DataId> {
        self.pointer_id.get(&pointer_value).cloned()
    }

    /// Insert `new_entry` into the [DataSection] and return its handle as [DataId].
    ///
    /// Inserting performs deduplication. If the equivalent [Entry] already exists
    /// (see [Entry::equiv]) the existing [DataId] is returned.
    ///
    /// Panics if the data section layout is frozen (see [Self::freeze_layout]).
    pub(crate) fn insert_data_value(&mut self, new_entry: Entry) -> DataId {
        assert!(
            self.frozen_offsets.is_none(),
            "cannot insert entries into the data section once its layout is frozen",
        );

        // If there is an identical data value, use the same id.
        let (value_pairs, kind) = match new_entry.name {
            EntryName::NonConfigurable => (
                &mut self.non_configurables,
                DataSectionRegion::NonConfigurables,
            ),
            EntryName::Configurable(_) => {
                (&mut self.configurables, DataSectionRegion::Configurables)
            }
        };
        match value_pairs.iter().position(|entry| entry.equiv(&new_entry)) {
            Some(num) => DataId {
                idx: num as u32,
                region: kind,
            },
            None => {
                value_pairs.push(new_entry);
                DataId {
                    idx: (value_pairs.len() - 1) as u32,
                    region: kind,
                }
            }
        }
    }

    /// If the stored data is [Datum::Word], return the inner value.
    pub(crate) fn get_data_word(&self, data_id: &DataId) -> Option<u64> {
        let value_pairs = match data_id.region {
            DataSectionRegion::NonConfigurables => &self.non_configurables,
            DataSectionRegion::Pointers => &self.pointers,
            DataSectionRegion::Configurables => &self.configurables,
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
