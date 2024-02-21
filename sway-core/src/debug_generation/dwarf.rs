use std::fs::File;

use gimli::write::{
    self, Address, DebugLine, DebugLineStrOffsets, DebugStrOffsets, Dwarf, EndianVec, LineProgram,
    LineString,
};
use gimli::{Encoding, Format, LineEncoding, LittleEndian};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{SourceEngine, Span};

use crate::asm_generation::instruction_set::InstructionSet;

use crate::source_map::SourceMap;
use crate::CompiledAsm;

use super::DebugInfo;

pub fn generate_debug_info(
    handler: &Handler,
    asm: &CompiledAsm,
    source_map: &mut SourceMap,
    source_engine: &SourceEngine,
) -> Result<DebugInfo, ErrorEmitted> {
    // Lets gather all the files used by the allocated ops in the compiled assembly.
    let source_spans = gather_source_file_spans(asm);

    let source_ids: Vec<_> = source_spans
        .iter()
        .map(|span| span.source_id())
        .filter_map(|source_id| match source_id {
            Some(source_id) => source_engine.get_file_name(source_id),
            None => None,
        } )
        .collect();

    let dir1 = &b"dir1"[..];
    let file1 = &b"file1"[..];
    let file2 = &b"file2"[..];
    let convert_address = &|address| Some(Address::Constant(address));

    let debug_line_str_offsets = DebugLineStrOffsets::none();
    let debug_str_offsets = DebugStrOffsets::none();

    for &version in &[2, 3, 4, 5] {
        for &address_size in &[4, 8] {
            for &format in &[Format::Dwarf64] {
                let encoding = Encoding {
                    format,
                    version,
                    address_size,
                };
                let line_base = -5;
                let line_range = 14;
                let neg_line_base = (-line_base) as u8;
                let mut program = LineProgram::new(
                    encoding,
                    LineEncoding {
                        line_base,
                        line_range,
                        ..Default::default()
                    },
                    LineString::String(dir1.to_vec()),
                    LineString::String(file1.to_vec()),
                    None,
                );
                let dir_id = program.default_directory();
                program.add_file(LineString::String(file1.to_vec()), dir_id, None);
                let file_id = program.add_file(LineString::String(file2.to_vec()), dir_id, None);

                // Test sequences.
                {
                    let mut program = program.clone();
                    let address = Address::Constant(0x12);
                    program.begin_sequence(Some(address));
                }

                // Create a base program.
                program.begin_sequence(None);
                //program.row.line = 0x1000;

                program.generate_row();
                let current_row = program.row();
                current_row.line = 100;

                program.end_sequence(0);

                let mut debug_line = DebugLine::from(EndianVec::new(LittleEndian));
                //let mut debug_line_offsets = Vec::new();

                program.write(
                    &mut debug_line,
                    encoding,
                    &debug_line_str_offsets,
                    &debug_str_offsets,
                );
            }
        }
    }

    let mut dwarf = Dwarf::new();
    // Write to new sections
    let mut write_sections = write::Sections::new(EndianVec::new(LittleEndian));
    dwarf
        .write(&mut write_sections)
        .expect("Should write DWARF information");

    let mut file = File::create("test.debug").unwrap();
    write_sections
        .for_each(|section_id, endian_vec| {
            use std::io::prelude::*;

            println!("{:?}", section_id);
            let section_name_bytes = section_id.name().as_bytes();
            file.write_all(&section_name_bytes.len().to_le_bytes())
                .unwrap();
            file.write_all(section_name_bytes).unwrap();

            let data = endian_vec.slice();
            file.write_all(&data.len().to_le_bytes()).unwrap();

            Ok::<(), ()>(())
        })
        .unwrap();

    Ok(DebugInfo {})
}

fn gather_source_file_spans(asm: &CompiledAsm) -> Vec<Span> {
    // Gather the set of spans used by all the allocated ops.
    match &asm.0.program_section {
        InstructionSet::Fuel { ops } => ops
            .iter()
            .filter_map(|op| match op.owning_span.as_ref() {
                Some(span) => Some(dbg!(span.clone())),
                None => {
                    dbg!("Instruction provides no span", op);
                    None
                }
            })
            .collect::<Vec<_>>(),
        InstructionSet::Evm { .. } => {
            unreachable!()
        }
        InstructionSet::MidenVM { .. } => {
            unreachable!()
        }
    }
}
