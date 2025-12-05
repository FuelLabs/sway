use crate::utils::bytecode::parse_bytecode_to_instructions;
use clap::Parser;
use forc_types::ForcResult;
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use tracing::info;

forc_types::cli_examples! {
    crate::cli::Opt {
        [Parse bytecode => "forc parse-bytecode <PATH>"]
    }
}

/// Parse bytecode file into a debug format.
#[derive(Debug, Parser)]
#[clap(bin_name = "forc parse-bytecode", version, after_help = help())]
pub(crate) struct Command {
    file_path: String,
}

pub(crate) fn exec(command: Command) -> ForcResult<()> {
    let instructions = parse_bytecode_to_instructions(&command.file_path)?;

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
            Err(fuel_asm::InvalidOpcode) if word_ix == 4 || word_ix == 5 => {
                let parsed_raw = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
                format!(
                    "configurables offset {} ({})",
                    if word_ix == 4 { "lo" } else { "hi" },
                    parsed_raw
                )
            }
            Ok(_) | Err(fuel_asm::InvalidOpcode) => "".into(),
        };
        table.add_row(Row::new(vec![
            TableCell::builder(word_ix)
                .col_span(1)
                .alignment(Alignment::Right)
                .build(),
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
