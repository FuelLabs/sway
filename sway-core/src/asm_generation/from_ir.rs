use super::{
    asm_builder::{AsmBuilder, AsmBuilderResult},
    evm::EvmAsmBuilder,
    finalized_asm::{check_invalid_opcodes, FinalizedAsm},
    fuel::{
        data_section::{DataId, DataSection},
        fuel_asm_builder::FuelAsmBuilder,
        register_sequencer::RegisterSequencer,
    },
    programs::{AbstractEntry, AbstractProgram, FinalProgram, ProgramKind},
    MidenVMAsmBuilder,
};

use crate::{BuildConfig, BuildTarget, ExperimentalFlags};

use sway_error::handler::{ErrorEmitted, Handler};
use sway_ir::{Context, Kind, Module};

pub fn compile_ir_to_asm(
    handler: &Handler,
    ir: &Context,
    build_config: Option<&BuildConfig>,
) -> Result<FinalizedAsm, ErrorEmitted> {
    // Eventually when we get this 'correct' with no hacks we'll want to compile all the modules
    // separately and then use a linker to connect them.  This way we could also keep binary caches
    // of libraries and link against them, rather than recompile everything each time.  For now we
    // assume there is one module.
    assert!(ir.module_iter().count() == 1);

    let module = ir.module_iter().next().unwrap();
    let final_program =
        compile_module_to_asm(handler, RegisterSequencer::new(), ir, module, build_config)?;

    let final_asm = final_program.finalize();

    check_invalid_opcodes(handler, &final_asm)?;

    Ok(final_asm)
}

fn compile_module_to_asm(
    handler: &Handler,
    reg_seqr: RegisterSequencer,
    context: &Context,
    module: Module,
    build_config: Option<&BuildConfig>,
) -> Result<FinalProgram, ErrorEmitted> {
    let kind = match module.get_kind(context) {
        Kind::Contract => ProgramKind::Contract,
        Kind::Library => ProgramKind::Library,
        Kind::Predicate => ProgramKind::Predicate,
        Kind::Script => ProgramKind::Script,
    };

    let build_target = match build_config {
        Some(cfg) => cfg.build_target,
        None => BuildTarget::default(),
    };

    let mut builder: Box<dyn AsmBuilder> = match build_target {
        BuildTarget::Fuel => Box::new(FuelAsmBuilder::new(
            kind,
            DataSection::default(),
            reg_seqr,
            context,
        )),
        BuildTarget::EVM => Box::new(EvmAsmBuilder::new(kind, context)),
        BuildTarget::MidenVM => Box::new(MidenVMAsmBuilder::new(kind, context)),
    };

    let mut fallback_fn = None;

    // Pre-create labels for all functions before we generate other code, so we can call them
    // before compiling them if needed.
    for func in module.function_iter(context) {
        let (start, _) = builder.func_to_labels(&func);
        if func.is_fallback(context) {
            fallback_fn = Some(start);
        }
    }

    for function in module.function_iter(context) {
        builder.compile_function(handler, function)?;
    }

    // Get the compiled result and massage a bit for the AbstractProgram.
    let result = builder.finalize();
    let final_program = match result {
        AsmBuilderResult::Fuel(result) => {
            let (data_section, reg_seqr, entries, non_entries) = result;
            let entries = entries
                .into_iter()
                .map(|(func, label, ops, test_decl_ref)| {
                    let selector = func.get_selector(context);
                    let name = func.get_name(context).to_string();
                    AbstractEntry {
                        test_decl_ref,
                        selector,
                        label,
                        ops,
                        name,
                    }
                })
                .collect();

            let abstract_program = AbstractProgram::new(
                kind,
                data_section,
                entries,
                non_entries,
                reg_seqr,
                ExperimentalFlags {
                    new_encoding: context.experimental.new_encoding,
                },
            );

            // Compiled dependencies will not have any content and we
            // do not want to display their empty ASM structures.
            // If printing ASM is requested, we want to emit the
            // actual ASMs generated for the whole program.
            let program_has_content = !abstract_program.is_empty();

            if build_config
                .map(|cfg| cfg.print_asm.virtual_abstract && program_has_content)
                .unwrap_or(false)
            {
                println!(";; ASM: Virtual abstract program");
                println!("{abstract_program}\n");
            }

            let allocated_program = abstract_program
                .into_allocated_program(fallback_fn)
                .map_err(|e| handler.emit_err(e))?;

            if build_config
                .map(|cfg| cfg.print_asm.allocated_abstract && program_has_content)
                .unwrap_or(false)
            {
                println!(";; ASM: Allocated abstract program");
                println!("{allocated_program}");
            }

            let final_program = allocated_program
                .into_final_program()
                .map_err(|e| handler.emit_err(e))?;

            if build_config
                .map(|cfg| cfg.print_asm.r#final && program_has_content)
                .unwrap_or(false)
            {
                println!(";; ASM: Final program");
                println!("{final_program}");
            }

            final_program
        }
        AsmBuilderResult::Evm(result) => FinalProgram::Evm {
            ops: result.ops,
            abi: result.abi,
        },
        AsmBuilderResult::MidenVM(result) => FinalProgram::MidenVM { ops: result.ops },
    };

    Ok(final_program)
}

// -------------------------------------------------------------------------------------------------

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
