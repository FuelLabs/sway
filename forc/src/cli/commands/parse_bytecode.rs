use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs::{self, File};
use std::io::Read;
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};

/// Parse bytecode file into a debug format.
#[derive(Debug, Parser)]
pub(crate) struct Command {
    file_path: String,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    let mut f = File::open(&command.file_path)
        .map_err(|_| anyhow!("{}: file not found", command.file_path))?;
    let metadata = fs::metadata(&command.file_path)
        .map_err(|_| anyhow!("{}: file not found", command.file_path))?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    let mut instructions = vec![];

    for i in (0..buffer.len()).step_by(4) {
        let i = i as usize;
        let raw = &buffer[i..i + 4];
        unsafe {
            let op = fuel_asm::Opcode::from_bytes_unchecked(raw);
            instructions.push((raw, op));
        };
    }
    let mut table = term_table::Table::new();
    table.separate_rows = false;
    table.add_row(Row::new(vec![
        TableCell::new("half-word"),
        TableCell::new("byte"),
        TableCell::new("op"),
        TableCell::new("raw"),
        TableCell::new("notes"),
    ]));
    table.style = term_table::TableStyle::empty();
    for (word_ix, instruction) in instructions.iter().enumerate() {
        use fuel_asm::Opcode::*;
        let notes = match instruction.1 {
            JI(num) => format!("conditionally jumps to byte {}", num * 4),
            JNEI(_, _, num) => format!("conditionally jumps to byte {}", num * 4),
            Undefined if word_ix == 2 || word_ix == 3 => {
                let parsed_raw = u32::from_be_bytes([
                    instruction.0[0],
                    instruction.0[1],
                    instruction.0[2],
                    instruction.0[3],
                ]);
                format!(
                    "data section offset {} ({})",
                    if word_ix == 2 { "lo" } else { "hi" },
                    parsed_raw
                )
            }
            _ => "".into(),
        };
        table.add_row(Row::new(vec![
            TableCell::new_with_alignment(word_ix, 1, Alignment::Right),
            TableCell::new(word_ix * 4),
            TableCell::new(format!("{:?}", instruction.1)),
            TableCell::new(format!("{:?}", instruction.0)),
            TableCell::new(notes),
        ]));
    }

    println!("{}", table.render());

    Ok(())
}
