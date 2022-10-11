use super::{
    asm_builder::AsmBuilder,
    checks::{check_invalid_opcodes, check_invalid_return},
    finalized_asm::FinalizedAsm,
    programs::{AbstractProgram, ProgramKind},
    register_sequencer::RegisterSequencer,
    DataId, DataSection,
};

use crate::{err, ok, BuildConfig, CompileResult, CompileWarning};

use sway_error::error::CompileError;
use sway_ir::*;

pub fn compile_ir_to_asm(
    ir: &Context,
    build_config: Option<&BuildConfig>,
) -> CompileResult<FinalizedAsm> {
    // Eventually when we get this 'correct' with no hacks we'll want to compile all the modules
    // separately and then use a linker to connect them.  This way we could also keep binary caches
    // of libraries and link against them, rather than recompile everything each time.  For now we
    // assume there is one module.
    assert!(ir.module_iter().count() == 1);

    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();

    let module = ir.module_iter().next().unwrap();

    check!(
        check_invalid_return(ir, &module),
        return err(warnings, errors),
        warnings,
        errors
    );

    let abstract_program = check!(
        compile_module_to_asm(RegisterSequencer::new(), ir, module),
        return err(warnings, errors),
        warnings,
        errors
    );

    if build_config
        .map(|cfg| cfg.print_intermediate_asm)
        .unwrap_or(false)
    {
        println!(";; --- ABSTRACT VIRTUAL PROGRAM ---\n");
        println!("{abstract_program}\n");
    }

    let allocated_program = abstract_program.into_allocated_program();

    if build_config
        .map(|cfg| cfg.print_intermediate_asm)
        .unwrap_or(false)
    {
        println!(";; --- ABSTRACT ALLOCATED PROGRAM ---\n");
        println!("{allocated_program}");
    }

    let final_program = allocated_program.into_final_program();

    if build_config
        .map(|cfg| cfg.print_finalized_asm)
        .unwrap_or(false)
    {
        println!(";; --- FINAL PROGRAM ---\n");
        println!("{final_program}");
    }

    let final_asm = final_program.finalize();

    check!(
        check_invalid_opcodes(&final_asm),
        return err(warnings, errors),
        warnings,
        errors
    );

    ok(final_asm, warnings, errors)
}

fn compile_module_to_asm(
    reg_seqr: RegisterSequencer,
    context: &Context,
    module: Module,
) -> CompileResult<AbstractProgram> {
    let mut builder = AsmBuilder::new(DataSection::default(), reg_seqr, context);

    // Pre-create labels for all functions before we generate other code, so we can call them
    // before compiling them if needed.
    for func in module.function_iter(context) {
        builder.func_to_labels(&func);
    }

    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for function in module.function_iter(context) {
        check!(
            builder.compile_function(function),
            return err(warnings, errors),
            warnings,
            errors
        );
    }

    // Get the compiled result and massage a bit for the AbstractProgram.
    let (data_section, reg_seqr, entries, non_entries) = builder.finalize();
    let entries = entries
        .into_iter()
        .map(|(func, label, ops)| {
            let selector = func.get_selector(context);
            (selector, label, ops)
        })
        .collect();
    let kind = match module.get_kind(context) {
        Kind::Contract => ProgramKind::Contract,
        Kind::Script => ProgramKind::Script,
        Kind::Library | Kind::Predicate => todo!("libraries and predicates coming soon!"),
    };

    ok(
        AbstractProgram::new(kind, data_section, entries, non_entries, reg_seqr),
        warnings,
        errors,
    )
}

// -------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! size_bytes_in_words {
    ($bytes_expr: expr) => {
        ($bytes_expr + 7) / 8
    };
}

// This is a mouthful...
#[macro_export]
macro_rules! size_bytes_round_up_to_word_alignment {
    ($bytes_expr: expr) => {
        ($bytes_expr + 7) - (($bytes_expr + 7) % 8)
    };
}

// NOTE: For stack storage we need to be aware:
// - sizes are in bytes; CFEI reserves in bytes.
// - offsets are in 64-bit words; LW/SW reads/writes to word offsets. XXX Wrap in a WordOffset struct.

#[derive(Clone, Debug)]
pub(super) enum Storage {
    Data(DataId), // Const storage in the data section.
    Stack(u64), // Storage in the runtime stack starting at an absolute word offset.  Essentially a global.
}

pub enum StateAccessType {
    Read,
    Write,
}

pub(crate) fn ir_type_size_in_bytes(context: &Context, ty: &Type) -> u64 {
    match ty {
        Type::Unit | Type::Bool | Type::Uint(_) | Type::Pointer(_) => 8,
        Type::B256 => 32,
        Type::String(n) => size_bytes_round_up_to_word_alignment!(n),
        Type::Array(aggregate) => {
            if let AggregateContent::ArrayType(el_ty, cnt) = aggregate.get_content(context) {
                cnt * ir_type_size_in_bytes(context, el_ty)
            } else {
                unreachable!("Wrong content for array.")
            }
        }
        Type::Struct(aggregate) => {
            if let AggregateContent::FieldTypes(field_tys) = aggregate.get_content(context) {
                // Sum up all the field sizes.
                field_tys
                    .iter()
                    .map(|field_ty| ir_type_size_in_bytes(context, field_ty))
                    .sum()
            } else {
                unreachable!("Wrong content for struct.")
            }
        }
        Type::Union(aggregate) => {
            if let AggregateContent::FieldTypes(field_tys) = aggregate.get_content(context) {
                // Find the max size for field sizes.
                field_tys
                    .iter()
                    .map(|field_ty| ir_type_size_in_bytes(context, field_ty))
                    .max()
                    .unwrap_or(0)
            } else {
                unreachable!("Wrong content for union.")
            }
        }
    }
}

// Aggregate (nested) field offset in words and size in bytes.
pub(crate) fn aggregate_idcs_to_field_layout(
    context: &Context,
    ty: &Type,
    idcs: &[u64],
) -> ((u64, u64), Type) {
    idcs.iter()
        .fold(((0, 0), *ty), |((offs, _), ty), idx| match ty {
            Type::Struct(aggregate) => {
                let idx = *idx as usize;
                let field_types = &aggregate.get_content(context).field_types();
                let field_type = field_types[idx];
                let field_offs_in_bytes = field_types
                    .iter()
                    .take(idx)
                    .map(|field_ty| ir_type_size_in_bytes(context, field_ty))
                    .sum::<u64>();
                let field_size_in_bytes = ir_type_size_in_bytes(context, &field_type);

                (
                    (
                        offs + size_bytes_in_words!(field_offs_in_bytes),
                        field_size_in_bytes,
                    ),
                    field_type,
                )
            }

            Type::Union(aggregate) => {
                let idx = *idx as usize;
                let field_type = aggregate.get_content(context).field_types()[idx];
                let union_size_in_bytes = ir_type_size_in_bytes(context, &ty);
                let field_size_in_bytes = ir_type_size_in_bytes(context, &field_type);

                // The union fields are at offset (union_size - variant_size) due to left padding.
                (
                    (
                        offs + size_bytes_in_words!(union_size_in_bytes - field_size_in_bytes),
                        field_size_in_bytes,
                    ),
                    field_type,
                )
            }

            _otherwise => panic!("Attempt to access field in non-aggregate."),
        })
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use sway_ir::parser::parse;

    use std::path::PathBuf;

    #[test]
    fn ir_to_asm_tests() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dir: PathBuf = format!("{}/tests/ir_to_asm", manifest_dir).into();
        for entry in std::fs::read_dir(dir).unwrap() {
            // We're only interested in the `.sw` files here.
            let path = entry.unwrap().path();
            match path.extension().unwrap().to_str() {
                Some("ir") => {
                    //
                    // Run the tests!
                    //
                    println!("---- IR To ASM: {:?} ----", path);
                    test_ir_to_asm(path);
                }
                Some("asm") | Some("disabled") => (),
                _ => panic!(
                    "File with invalid extension in tests dir: {:?}",
                    path.file_name().unwrap_or(path.as_os_str())
                ),
            }
        }
    }

    fn test_ir_to_asm(mut path: PathBuf) {
        let input_bytes = std::fs::read(&path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        path.set_extension("asm");

        let expected_bytes = std::fs::read(&path).unwrap();
        let expected = String::from_utf8_lossy(&expected_bytes);

        let ir = parse(&input).expect("parsed ir");
        let asm_result = compile_ir_to_asm(&ir, None);

        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let asm = asm_result.unwrap(&mut warnings, &mut errors);
        assert!(warnings.is_empty() && errors.is_empty());

        let asm_script = format!("{}", asm);
        if asm_script != expected {
            print!(
                "{}\n{}",
                path.display(),
                prettydiff::diff_lines(&expected, &asm_script)
            );
            panic!();
        }
    }
}

// =================================================================================================
