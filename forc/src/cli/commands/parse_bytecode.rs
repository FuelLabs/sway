use std::convert::TryInto;
use std::fs::{self, File};
use std::io::Read;
use structopt::{self, StructOpt};

#[derive(Debug, StructOpt)]
pub(crate) struct Command {
    file_path: String,
}

/// Parses the bytecode into a debug format.
pub(crate) fn exec(command: Command) -> Result<(), String> {
    let mut f = File::open(&command.file_path)
        .map_err(|e| format!("{}: file not found", command.file_path))?;
    let metadata = fs::metadata(&command.file_path)
        .map_err(|e| format!("{}: file not found", command.file_path))?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    let first_jump = fuel_asm::Opcode::from_bytes_unchecked(buffer[0..5].try_into().unwrap());
    let data_section_offset = u64::from_be_bytes(
        buffer[5..13]
            .try_into()
            .map_err(|e| format!("error while reading data section offset: {}", e))?,
    );
    let mut instructions = vec![first_jump];

    for i in (13..data_section_offset).step_by(4) {
        let i = i as usize;
        instructions.push(fuel_asm::Opcode::from_bytes_unchecked(
            buffer[i..i + 4].try_into().unwrap(),
        ));
    }

    dbg!(instructions);

    Ok(())
}
