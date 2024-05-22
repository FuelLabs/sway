pub(crate) mod compiler_constants;
pub(crate) mod data_section;
pub(crate) mod register_allocator;

pub(super) mod abstract_instruction_set;
pub(super) mod allocated_abstract_instruction_set;
pub(super) mod checks;
pub(super) mod fuel_asm_builder;
pub(super) mod register_sequencer;

pub(super) mod programs;

mod globals_section;

mod analyses;
mod functions;
mod optimizations;
