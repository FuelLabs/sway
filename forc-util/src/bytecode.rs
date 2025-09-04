use anyhow::anyhow;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

// The index of the beginning of the half-word (4 bytes) that contains the configurables section offset.
const CONFIGURABLES_OFFSET_INSTR_LO: usize = 4;
// The index of the end of the half-word (4 bytes) that contains the configurables section offset.
const CONFIGURABLES_OFFSET_INSTR_HI: usize = 5;
// The count of the beginning half-words that contain the configurables section offset.
const CONFIGURABLES_OFFSET_PREAMBLE: usize = CONFIGURABLES_OFFSET_INSTR_HI + 1;

/// A tuple of an instruction and its corresponding bytes. Useful when needing to access the raw bytes
/// of an instruction that is parsed as [fuel_asm::InvalidOpcode], such as metadata in the preamble.
pub type InstructionWithBytes = (
    Result<fuel_asm::Instruction, fuel_asm::InvalidOpcode>,
    Vec<u8>,
);

/// An iterator over each [fuel_asm::Instruction] or [fuel_asm::InvalidOpcode] with its corresponding bytes.
pub struct InstructionWithBytesIterator {
    buf_reader: BufReader<File>,
}

impl InstructionWithBytesIterator {
    /// Return a new iterator for each instruction parsed from raw bytes.
    pub fn new(buf_reader: BufReader<File>) -> Self {
        InstructionWithBytesIterator { buf_reader }
    }
}

impl Iterator for InstructionWithBytesIterator {
    type Item = InstructionWithBytes;

    fn next(&mut self) -> Option<InstructionWithBytes> {
        let mut buffer = [0; fuel_asm::Instruction::SIZE];
        // Read the next instruction into the buffer
        match self.buf_reader.read_exact(&mut buffer) {
            Ok(_) => fuel_asm::from_bytes(buffer)
                .next()
                .map(|inst| (inst, buffer.to_vec())),
            Err(_) => None,
        }
    }
}

/// Parses a bytecode file into an iterator of instructions and their corresponding bytes.
pub fn parse_bytecode_to_instructions<P>(path: P) -> anyhow::Result<InstructionWithBytesIterator>
where
    P: AsRef<Path> + Clone,
{
    let f = File::open(path.clone())
        .map_err(|_| anyhow!("{}: file not found", path.as_ref().to_string_lossy()))?;
    let buf_reader = BufReader::new(f);

    Ok(InstructionWithBytesIterator::new(buf_reader))
}

/// Gets the bytecode ID from a bytecode file. The bytecode ID is the hash of the bytecode after removing the
/// condigurables section, if any.
pub fn get_bytecode_id<P>(path: P) -> anyhow::Result<String>
where
    P: AsRef<Path> + Clone,
{
    let mut instructions = parse_bytecode_to_instructions(path.clone())?;

    // Collect the first six instructions into a temporary vector
    let mut first_six_instructions = Vec::with_capacity(CONFIGURABLES_OFFSET_PREAMBLE);
    for _ in 0..CONFIGURABLES_OFFSET_PREAMBLE {
        if let Some(instruction) = instructions.next() {
            first_six_instructions.push(instruction);
        } else {
            return Err(anyhow!("Incomplete bytecode"));
        }
    }

    let (lo_instr, low_raw) = &first_six_instructions[CONFIGURABLES_OFFSET_INSTR_LO];
    let (hi_instr, hi_raw) = &first_six_instructions[CONFIGURABLES_OFFSET_INSTR_HI];

    if let Err(fuel_asm::InvalidOpcode) = lo_instr {
        if let Err(fuel_asm::InvalidOpcode) = hi_instr {
            // Now assemble the configurables offset.
            let configurables_offset = usize::from_be_bytes([
                low_raw[0], low_raw[1], low_raw[2], low_raw[3], hi_raw[0], hi_raw[1], hi_raw[2],
                hi_raw[3],
            ]);

            // Hash the first six instructions
            let mut hasher = Sha256::new();
            for (_, raw) in first_six_instructions {
                hasher.update(raw);
            }

            // Continue hashing the remaining instructions up to the configurables section offset.
            instructions
                .take(
                    configurables_offset / fuel_asm::Instruction::SIZE
                        - CONFIGURABLES_OFFSET_PREAMBLE,
                ) // Minus 6 because we already hashed the first six
                .for_each(|(_, raw)| {
                    hasher.update(raw);
                });

            let hash_result = hasher.finalize();
            let bytecode_id = format!("{hash_result:x}");
            return Ok(bytecode_id);
        }
    }

    Err(anyhow!("Configurables section offset not found"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_bytecode_id_happy() {
        // These binary files were generated from `examples/configurable_constants` and `examples/counter`
        // using `forc build` and `forc build --release` respectively.
        let bytecode_id: String =
            get_bytecode_id("tests/fixtures/bytecode/debug-counter.bin").expect("bytecode id");
        assert_eq!(
            bytecode_id,
            "e65aa988cae1041b64dc2d85e496eed0e8a1d8105133bd313c17645a1859d53b".to_string()
        );

        let bytecode_id =
            get_bytecode_id("tests/fixtures/bytecode/release-counter.bin").expect("bytecode id");
        assert_eq!(
            bytecode_id,
            "42ae8352cbc892d7c7621f1d6fb42b072a08ba5968508d49f54991668d4ea141".to_string()
        );

        let bytecode_id =
            get_bytecode_id("tests/fixtures/bytecode/debug-configurable_constants.bin")
                .expect("bytecode id");
        assert_eq!(
            bytecode_id,
            "babc3d9dcac8d48dee1e5aeb3340ff098d3c1ab8b0a28341d9291d8ff757199e".to_string()
        );

        let bytecode_id =
            get_bytecode_id("tests/fixtures/bytecode/release-configurable_constants.bin")
                .expect("bytecode id");
        assert_eq!(
            bytecode_id,
            "2adfb515b66763fd29391bdba012921d045a0be83d89be5492bcaacc429695e9".to_string()
        );
    }

    #[test]
    fn test_get_bytecode_id_missing_configurable_offset() {
        // This bytecode file was generated from `examples/configurable_constants` using an older version of the
        // compiler that did not include the configurables section offset in the preamble.
        let result = get_bytecode_id(
            "tests/fixtures/bytecode/debug-configurable_constants-missing-offset.bin",
        );
        assert_eq!(
            result.unwrap_err().to_string().as_str(),
            "Configurables section offset not found"
        );
    }

    #[test]
    fn test_get_bytecode_id_bad_path() {
        let result = get_bytecode_id("tests/fixtures/bytecode/blahblahblahblah.bin");
        assert_eq!(
            result.unwrap_err().to_string().as_str(),
            "tests/fixtures/bytecode/blahblahblahblah.bin: file not found"
        );
    }
}
