use anyhow::anyhow;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

// The index of the beginning of the half-word (4 bytes) that contains the configurables section offset.
const CONFIGURABLES_OFFSET_INSTR_LO: usize = 4;
// The index of the end of the half-word (4 bytes) that contains the configurables section offset.
const CONFIGURABLES_OFFSET_INSTR_HI: usize = 5;

/// Parses a bytecode file into an iterator of instructions and their corresponding bytes.
pub fn parse_bytecode_to_instructions<P>(
    path: P,
) -> anyhow::Result<
    Vec<(
        Result<fuel_asm::Instruction, fuel_asm::InvalidOpcode>,
        Vec<u8>,
    )>,
>
where
    P: AsRef<Path> + Clone,
{
    let mut f = File::open(path.clone())
        .map_err(|_| anyhow!("{}: file not found", path.as_ref().to_string_lossy()))?;
    let metadata = fs::metadata(path.clone())
        .map_err(|_| anyhow!("{}: file not found", path.as_ref().to_string_lossy()))?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");

    let instructions = fuel_asm::from_bytes(buffer.clone()).zip(
        buffer
            .chunks(fuel_asm::Instruction::SIZE)
            .into_iter()
            .map(|chunk: &[u8]| chunk.to_vec()),
    );

    Ok(instructions.collect())
}

/// Gets the bytecode ID from a bytecode file. The bytecode ID is the hash of the bytecode after removing the
/// condigurables section, if any.
pub fn get_bytecode_id<P>(path: P) -> anyhow::Result<String>
where
    P: AsRef<Path> + Clone,
{
    let mut instructions = parse_bytecode_to_instructions(path.clone())?.into_iter();

    // Collect the first six instructions into a temporary vector
    let mut first_six_instructions = Vec::with_capacity(6);
    for _ in 0..CONFIGURABLES_OFFSET_INSTR_HI + 1 {
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
                .take(configurables_offset / 4 - 6) // Minus 6 because we already hashed the first six
                .for_each(|(_, raw)| {
                    hasher.update(raw);
                });

            let hash_result = hasher.finalize();
            let bytecode_id = format!("{:x}", hash_result);
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
