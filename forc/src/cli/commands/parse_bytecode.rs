use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs::{self, File};
use std::io::Read;
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use tracing::info;

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

    let instructions = fuel_asm::from_bytes(buffer.iter().cloned())
        .zip(buffer.chunks(fuel_asm::Instruction::SIZE));

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
    for (word_ix, (result, raw)) in instructions.enumerate() {
        use fuel_asm::Instruction;
        let notes = match result {
            Ok(Instruction::JI(ji)) => format!("jump to byte {}", u32::from(ji.imm24()) * 4),
            Ok(Instruction::JNEI(jnei)) => {
                format!("conditionally jump to byte {}", u32::from(jnei.imm12()) * 4)
            }
            Ok(Instruction::JNZI(jnzi)) => {
                format!("conditionally jump to byte {}", u32::from(jnzi.imm18()) * 4)
            }
            Err(fuel_asm::InvalidOpcode) if word_ix == 2 || word_ix == 3 => {
                let parsed_raw = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
                format!(
                    "data section offset {} ({})",
                    if word_ix == 2 { "lo" } else { "hi" },
                    parsed_raw
                )
            }
            Ok(_) | Err(fuel_asm::InvalidOpcode) => "".into(),
        };
        table.add_row(Row::new(vec![
            TableCell::new_with_alignment(word_ix, 1, Alignment::Right),
            TableCell::new(word_ix * 4),
            TableCell::new(match result {
                Ok(inst) => format!("{inst:?}"),
                Err(err) => format!("{err:?}"),
            }),
            TableCell::new(format!(
                "{:02x} {:02x} {:02x} {:02x}",
                raw[0], raw[1], raw[2], raw[3],
            )),
            TableCell::new(notes),
        ]));
    }

    info!("{}", table.render());

    Ok(())
}
