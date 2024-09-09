use std::fs::File;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

use gimli::write::{
    self, DebugLine, DebugLineStrOffsets, DebugStrOffsets, DwarfUnit, EndianVec, LineProgram,
    LineString,
};
use gimli::{BigEndian, Encoding, LineEncoding};
use sway_error::error::CompileError;
use sway_types::Span;

use crate::source_map::SourceMap;

use object::write::Object;

pub fn write_dwarf(
    source_map: &SourceMap,
    primary_dir: &Path,
    primary_src: &Path,
    out_file: &Path,
) -> Result<(), CompileError> {
    let encoding = gimli::Encoding {
        format: gimli::Format::Dwarf64,
        version: 5,
        address_size: 8,
    };

    let program = build_line_number_program(encoding, primary_dir, primary_src, source_map)?;

    program
        .write(
            &mut DebugLine::from(EndianVec::new(BigEndian)),
            encoding,
            &DebugLineStrOffsets::none(),
            &DebugStrOffsets::none(),
        )
        .map_err(|err| {
            sway_error::error::CompileError::InternalOwned(err.to_string(), Span::dummy())
        })?;

    let mut dwarf = DwarfUnit::new(encoding);
    dwarf.unit.line_program = program;
    // Write to new sections
    let mut debug_sections = write::Sections::new(EndianVec::new(BigEndian));
    dwarf.write(&mut debug_sections).map_err(|err| {
        sway_error::error::CompileError::InternalOwned(err.to_string(), Span::dummy())
    })?;

    let file = File::create(out_file).unwrap();
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
        sway_error::error::CompileError::InternalOwned(err.to_string(), Span::dummy())
    })?;

    Ok(())
}

fn build_line_number_program(
    encoding: Encoding,
    primary_dir: &Path,
    primary_src: &Path,
    source_map: &SourceMap,
) -> Result<LineProgram, CompileError> {
    let primary_src = primary_src.strip_prefix(primary_dir).map_err(|err| {
        sway_error::error::CompileError::InternalOwned(err.to_string(), Span::dummy())
    })?;
    let mut program = LineProgram::new(
        encoding,
        LineEncoding::default(),
        LineString::String(primary_dir.to_path_buf().into_os_string().into_vec()),
        LineString::String(primary_src.to_path_buf().into_os_string().into_vec()),
        None,
    );

    program.begin_sequence(Some(write::Address::Constant(0)));

    for (ix, span) in &source_map.map {
        let (path, span) = span.to_span(&source_map.paths, &source_map.dependency_paths);

        let dir = path
            .parent()
            .ok_or(sway_error::error::CompileError::InternalOwned(
                "Path doesn't have a proper prefix".to_string(),
                Span::dummy(),
            ))?;
        let file = path
            .file_name()
            .ok_or(sway_error::error::CompileError::InternalOwned(
                "Path doesn't have proper filename".to_string(),
                Span::dummy(),
            ))?;

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

    Ok(program)
}
