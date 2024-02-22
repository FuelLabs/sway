use std::fs::File;
use std::io::Write;

use gimli::write::{
    self, DebugLine, DebugLineStrOffsets, DebugStrOffsets, DwarfUnit, EndianVec, LineProgram,
    LineString,
};
use gimli::{BigEndian, LineEncoding};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{SourceEngine, Span};

use crate::asm_generation::instruction_set::InstructionSet;

use crate::source_map::SourceMap;
use crate::CompiledAsm;

use object::write::Object;

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
        })
        .collect();

    let dir1 = &b"dir1"[..];
    let file1 = &b"file1"[..];
    let file2 = &b"file2"[..];

    let debug_line_str_offsets = DebugLineStrOffsets::none();
    let debug_str_offsets = DebugStrOffsets::none();

    let encoding = gimli::Encoding {
        format: gimli::Format::Dwarf64,
        version: 5,
        address_size: 8,
    };
    let mut program = LineProgram::new(
        encoding,
        LineEncoding::default(),
        LineString::String(dir1.to_vec()),
        LineString::String(file1.to_vec()),
        None,
    );
    let dir_id = program.default_directory();
    program.add_file(LineString::String(file1.to_vec()), dir_id, None);
    program.add_file(LineString::String(file2.to_vec()), dir_id, None);

    // Create a base program.
    program.begin_sequence(None);
    //program.row.line = 0x1000;

    program.generate_row();
    let current_row = program.row();
    current_row.line = 100;

    program.end_sequence(0);

    let mut debug_line = DebugLine::from(EndianVec::new(BigEndian));
    //let mut debug_line_offsets = Vec::new();

    program
        .write(
            &mut debug_line,
            encoding,
            &debug_line_str_offsets,
            &debug_str_offsets,
        )
        .map_err(|err| {
            handler.emit_err(sway_error::error::CompileError::InternalOwned(
                err.to_string(),
                Span::dummy(),
            ))
        })?;

    let mut dwarf = DwarfUnit::new(encoding);
    dwarf.unit.line_program = program;
    // Write to new sections
    let mut debug_sections = write::Sections::new(EndianVec::new(BigEndian));
    dwarf
        .write(&mut debug_sections)
        .expect("Should write DWARF information");

    let file = File::create("test.debug").unwrap();
    let mut obj = Object::new(
        object::BinaryFormat::Elf,
        object::Architecture::X86_64,
        object::Endianness::Big,
    );
    debug_sections
        .for_each(|section_id, data| {
            let sec = obj.add_section(
                [].into(),
                section_id.name().into(),
                object::SectionKind::Other,
            );
            obj.set_section_data(sec, data.clone().into_vec(), 8);
            Ok::<(), ()>(())
        })
        .unwrap();

    obj.write_stream(file).map_err(|err| {
        handler.emit_err(sway_error::error::CompileError::InternalOwned(
            err.to_string(),
            Span::dummy(),
        ))
    })?;

    Ok(DebugInfo {})
}

fn gather_source_file_spans(asm: &CompiledAsm) -> Vec<Span> {
    // Gather the set of spans used by all the allocated ops.
    match &asm.0.program_section {
        InstructionSet::Fuel { ops } => ops
            .iter()
            .filter_map(|op| match op.owning_span.as_ref() {
                Some(span) => Some(span.clone()),
                None => None,
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
