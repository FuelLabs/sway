use super::{
    asm_builder::AsmBuilder,
    evm::EvmAsmBuilder,
    finalized_asm::{check_invalid_opcodes, FinalizedAsm},
    fuel::{
        data_section::{DataId, DataSection},
        fuel_asm_builder::FuelAsmBuilder,
        register_sequencer::RegisterSequencer,
    },
};
use crate::{asm_generation::ProgramKind, BuildConfig, BuildTarget};

use crate::asm_lang::VirtualImmediate18;

use sway_error::handler::{ErrorEmitted, Handler};
use sway_ir::{Context, Kind, Module};

pub fn compile_ir_context_to_finalized_asm(
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

    let reg_seqr = RegisterSequencer::new();
    let kind = match module.get_kind(ir) {
        Kind::Contract => ProgramKind::Contract,
        Kind::Library => ProgramKind::Library,
        Kind::Predicate => ProgramKind::Predicate,
        Kind::Script => ProgramKind::Script,
    };

    let build_target = match build_config {
        Some(cfg) => cfg.build_target,
        None => BuildTarget::default(),
    };

    let finalized_asm = match build_target {
        BuildTarget::Fuel | BuildTarget::Native => compile(
            handler,
            ir,
            module,
            build_config,
            FuelAsmBuilder::new(kind, DataSection::default(), reg_seqr, ir),
        ),
        BuildTarget::EVM => compile(
            handler,
            ir,
            module,
            build_config,
            EvmAsmBuilder::new(kind, ir),
        ),
        BuildTarget::Polkavm => compile(
            handler,
            ir,
            module,
            build_config,
            FuelAsmBuilder::new(kind, DataSection::default(), reg_seqr, ir),
        ),
    }?;

    check_invalid_opcodes(handler, &finalized_asm)?;

    Ok(finalized_asm)
}

fn compile(
    handler: &Handler,
    context: &Context,
    module: Module,
    build_config: Option<&BuildConfig>,
    mut builder: impl AsmBuilder,
) -> Result<FinalizedAsm, ErrorEmitted> {
    let mut fallback_fn = None;

    // Pre-create labels for all functions before we generate other code, so we can call them
    // before compiling them if needed.
    for func in module.function_iter(context) {
        let (start, _) = builder.func_to_labels(&func);
        if func.is_fallback(context) {
            fallback_fn = Some(start);
        }
    }

    for config in module.iter_configs(context) {
        builder.compile_configurable(config);
    }

    for function in module.function_iter(context) {
        builder.compile_function(handler, function)?;
    }

    builder.finalize(handler, build_config, fallback_fn)
}

// -------------------------------------------------------------------------------------------------

// NOTE: For stack storage we need to be aware:
// - sizes are in bytes; CFEI reserves in bytes.
// - offsets are in 64-bit words; LW/SW reads/writes to word offsets. XXX Wrap in a WordOffset struct.

#[derive(Clone, Debug)]
pub(super) enum Storage {
    Data(DataId),              // Const storage in the data section.
    Stack(u64), // Storage in the runtime stack starting at an absolute word offset.  Essentially a global.
    Const(VirtualImmediate18), // An immediate value that can be moved to a register using MOVI.
}

pub enum StateAccessType {
    Read,
    Write,
}
