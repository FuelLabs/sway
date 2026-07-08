library;

pub mod types;

mod init_struct;
mod init_struct_sroa;
mod init_struct_from_branching;
mod init_struct_with_str;

mod init_array_repeat;
mod init_array_repeat_sroa;
mod init_array_not_repeat;
mod init_array_repeat_split_block_insert_loop;

mod init_tuple;

mod init_with_ref_mut_args;

mod init_mostly_zeroed;

mod init_empty_aggregates;

// Aggregates reducing to a single stored value (single-field/wrapper structs).
mod init_single_store;

// Aggregates containing enums.
mod init_enums;

// Additional mixed nesting constellations.
mod init_misc_constellations;

// Regression tests for issues found while developing the optimization.
mod issues;
