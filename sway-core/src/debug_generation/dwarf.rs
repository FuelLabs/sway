use std::fs::File;

use gimli::write::{
    self, DebugLine, DebugLineStrOffsets, DebugStrOffsets, DwarfUnit, EndianVec, LineProgram,
    LineString,
};
use gimli::{BigEndian, LineEncoding};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::source_map::SourceMap;

use object::write::Object;

pub fn generate_debug_info(handler: &Handler, source_map: &SourceMap) -> Result<(), ErrorEmitted> {
    // working directory
    let working_dir = &b"sway"[..];
    // primary source file
    let primary_src = &b"main.sw"[..];

    let encoding = gimli::Encoding {
        format: gimli::Format::Dwarf64,
        version: 5,
        address_size: 8,
    };
    let mut program = LineProgram::new(
        encoding,
        LineEncoding::default(),
        LineString::String(working_dir.to_vec()),
        LineString::String(primary_src.to_vec()),
        None,
    );

    build_line_number_program(source_map, &mut program)?;

    program
        .write(
            &mut DebugLine::from(EndianVec::new(BigEndian)),
            encoding,
            &DebugLineStrOffsets::none(),
            &DebugStrOffsets::none(),
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

    Ok(())
}

fn build_line_number_program(
    source_map: &SourceMap,
    program: &mut LineProgram,
) -> Result<(), ErrorEmitted> {
    program.begin_sequence(Some(write::Address::Constant(0)));

    for (ix, span) in &source_map.map {
        let (path, span) = span.to_span(&source_map.paths, &source_map.dependency_paths);

        let dir = path.parent().expect("Path doesn't have proper prefix");
        let file = path.file_name().expect("Path doesn't have proper filename");

        let dir_id = program.add_directory(LineString::String(
            dir.as_os_str().as_encoded_bytes().into(),
        ));
        let file_id = program.add_file(
            LineString::String(file.as_encoded_bytes().into()),
            dir_id,
            None,
        );

        program.generate_row();

        let current_row = program.row();
        current_row.line = span.start.line as u64;
        current_row.column = span.start.col as u64;
        current_row.address_offset = *ix as u64;
        current_row.file = file_id;
    }

    program.end_sequence(
        source_map
            .map
            .last_key_value()
            .map(|(key, _)| *key)
            .unwrap_or_default() as u64
            + 1,
    );
    Ok(())
}
