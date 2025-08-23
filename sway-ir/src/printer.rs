//! Print (or serialize) IR to human and machine readable text.
//!
//! This module implements a document based pretty-printer.  A couple of 3rd party pretty printing
//! crates were assessed but didn't seem to work as well as this simple version, which is quite
//! effective.

use std::collections::{BTreeMap, HashMap};

use sway_types::SourceEngine;

use crate::{
    asm::*,
    block::Block,
    constant::{ConstantContent, ConstantValue},
    context::Context,
    function::{Function, FunctionContent},
    instruction::{FuelVmInstruction, InstOp, Predicate, Register},
    metadata::{MetadataIndex, Metadatum},
    module::{Kind, ModuleContent},
    value::{Value, ValueContent, ValueDatum},
    AnalysisResult, AnalysisResultT, AnalysisResults, BinaryOpKind, BlockArgument, ConfigContent,
    IrError, Module, Pass, PassMutability, ScopedPass, UnaryOpKind,
};

#[derive(Debug)]
pub(crate) enum Doc {
    Empty,
    Space,
    Comma,

    Text(String),
    Line(Box<Doc>),

    Pair(Box<Doc>, Box<Doc>),

    List(Vec<Doc>),
    ListSep(Vec<Doc>, Box<Doc>),

    Parens(Box<Doc>),

    Indent(i64, Box<Doc>),
}

impl Doc {
    pub(crate) fn text<S: Into<String>>(s: S) -> Self {
        Doc::Text(s.into())
    }

    fn line(doc: Doc) -> Self {
        Doc::Line(Box::new(doc))
    }

    pub(crate) fn text_line<S: Into<String>>(s: S) -> Self {
        Doc::Line(Box::new(Doc::Text(s.into())))
    }

    fn indent(n: i64, doc: Doc) -> Doc {
        Doc::Indent(n, Box::new(doc))
    }

    fn list_sep(docs: Vec<Doc>, sep: Doc) -> Doc {
        Doc::ListSep(docs, Box::new(sep))
    }

    fn in_parens_comma_sep(docs: Vec<Doc>) -> Doc {
        Doc::Parens(Box::new(Doc::list_sep(docs, Doc::Comma)))
    }

    pub(crate) fn append(self, doc: Doc) -> Doc {
        match (&self, &doc) {
            (Doc::Empty, _) => doc,
            (_, Doc::Empty) => self,
            _ => Doc::Pair(Box::new(self), Box::new(doc)),
        }
    }

    fn and(self, doc: Doc) -> Doc {
        match doc {
            Doc::Empty => doc,
            _ => Doc::Pair(Box::new(self), Box::new(doc)),
        }
    }

    pub(crate) fn build(self) -> String {
        build_doc(self, 0)
    }
}

/// Pretty-print a whole [`Context`] to a string.
///
/// The output from this function must always be suitable for [crate::parser::parse].
pub fn to_string(context: &Context) -> String {
    context_print(context, &|_, doc| doc)
}

pub(crate) fn context_print(context: &Context, map_doc: &impl Fn(&Value, Doc) -> Doc) -> String {
    let mut md_namer = MetadataNamer::default();
    context
        .modules
        .iter()
        .fold(Doc::Empty, |doc, (_, module)| {
            doc.append(module_to_doc(context, &mut md_namer, module, map_doc))
        })
        .append(md_namer.to_doc(context))
        .build()
}

pub(crate) fn block_print(
    context: &Context,
    function: Function,
    block: Block,
    map_doc: &impl Fn(&Value, Doc) -> Doc,
) -> String {
    let mut md_namer = MetadataNamer::default();
    let mut namer = Namer::new(function);
    block_to_doc(context, &mut md_namer, &mut namer, &block, map_doc).build()
}

pub struct ModulePrinterResult;
impl AnalysisResultT for ModulePrinterResult {}

/// Pass to print a module to stdout.
pub fn module_printer_pass(
    context: &Context,
    _analyses: &AnalysisResults,
    module: Module,
) -> Result<AnalysisResult, IrError> {
    let mut md_namer = MetadataNamer::default();
    print!(
        "{}",
        module_to_doc(
            context,
            &mut md_namer,
            context.modules.get(module.0).unwrap(),
            &|_, doc| doc
        )
        .append(md_namer.to_doc(context))
        .build()
    );
    Ok(Box::new(ModulePrinterResult))
}

/// Print a module to stdout.
pub fn module_print(context: &Context, _analyses: &AnalysisResults, module: Module) {
    let mut md_namer = MetadataNamer::default();
    println!(
        "{}",
        module_to_doc(
            context,
            &mut md_namer,
            context.modules.get(module.0).unwrap(),
            &|_, doc| doc
        )
        .append(md_namer.to_doc(context))
        .build()
    );
}

/// Print a function to stdout.
pub fn function_print(context: &Context, function: Function) {
    let mut md_namer = MetadataNamer::default();
    println!(
        "{}",
        function_to_doc(
            context,
            &mut md_namer,
            &mut Namer::new(function),
            context.functions.get(function.0).unwrap(),
            &|_, doc| doc
        )
        .append(md_namer.to_doc(context))
        .build()
    );
}

/// Print an instruction to stdout.
pub fn instruction_print(context: &Context, ins_value: &Value) {
    let mut md_namer = MetadataNamer::default();
    let block = ins_value
        .get_instruction(context)
        .expect("Calling instruction printer on non-instruction value")
        .parent;
    let function = block.get_function(context);
    let mut namer = Namer::new(function);
    println!(
        "{}",
        instruction_to_doc(context, &mut md_namer, &mut namer, &block, ins_value).build()
    );
}

pub const MODULE_PRINTER_NAME: &str = "module-printer";

pub fn create_module_printer_pass() -> Pass {
    Pass {
        name: MODULE_PRINTER_NAME,
        descr: "Print module to stdout",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Analysis(module_printer_pass)),
    }
}

fn module_to_doc<'a>(
    context: &'a Context,
    md_namer: &mut MetadataNamer,
    module: &'a ModuleContent,
    map_doc: &impl Fn(&Value, Doc) -> Doc,
) -> Doc {
    Doc::line(Doc::Text(format!(
        "{} {{",
        match module.kind {
            Kind::Contract => "contract",
            Kind::Library => "library",
            Kind::Predicate => "predicate",
            Kind::Script => "script",
        }
    )))
    .append(Doc::indent(
        4,
        Doc::List(
            module
                .configs
                .values()
                .map(|value| config_to_doc(context, value, md_namer))
                .collect(),
        ),
    ))
    .append(if !module.configs.is_empty() {
        Doc::line(Doc::Empty)
    } else {
        Doc::Empty
    })
    .append(Doc::indent(
        4,
        Doc::List(
            module
                .global_variables
                .iter()
                .map(|(name, var)| {
                    let var_content = &context.global_vars[var.0];
                    let init_doc = match &var_content.initializer {
                        Some(const_val) => Doc::text(format!(
                            " = const {}",
                            const_val.get_content(context).as_lit_string(context)
                        )),
                        None => Doc::Empty,
                    };
                    let mut_string = if var_content.mutable { "mut " } else { "" };
                    Doc::line(
                        Doc::text(format!(
                            "{}global {} : {}",
                            mut_string,
                            name.join("::"),
                            var.get_inner_type(context).as_string(context),
                        ))
                        .append(init_doc),
                    )
                })
                .collect(),
        ),
    ))
    .append(if !module.global_variables.is_empty() {
        Doc::line(Doc::Empty)
    } else {
        Doc::Empty
    })
    .append(Doc::indent(
        4,
        Doc::List(
            module
                .storage_keys
                .iter()
                .map(|(name, storage_key)| {
                    let (slot, offset, field_id) = storage_key.get_parts(context);
                    Doc::line(
                        // If the storage key's path doesn't have struct field names,
                        // which is 99% of the time, we will display only the slot,
                        // to avoid clattering.
                        Doc::text(format!(
                            "storage_key {name} = 0x{slot:x}{}{}",
                            if offset != 0 || slot != field_id {
                                format!(" : {offset}")
                            } else {
                                "".to_string()
                            },
                            if slot != field_id {
                                format!(" : 0x{field_id:x}")
                            } else {
                                "".to_string()
                            },
                        ))
                    )
                })
                .collect(),
        ),
    ))
    .append(if !module.storage_keys.is_empty() {
        Doc::line(Doc::Empty)
    } else {
        Doc::Empty
    })
    .append(Doc::indent(
        4,
        Doc::list_sep(
            module
                .functions
                .iter()
                .map(|function| {
                    function_to_doc(
                        context,
                        md_namer,
                        &mut Namer::new(*function),
                        &context.functions[function.0],
                        map_doc,
                    )
                })
                .collect(),
            Doc::line(Doc::Empty),
        ),
    ))
    .append(Doc::text_line("}"))
}

fn config_to_doc(
    context: &Context,
    configurable: &ConfigContent,
    md_namer: &mut MetadataNamer,
) -> Doc {
    match configurable {
        ConfigContent::V0 {
            name,
            constant,
            opt_metadata,
            ..
        } => Doc::line(
            Doc::text(format!(
                "{} = config {}",
                name,
                constant.get_content(context).as_lit_string(context)
            ))
            .append(md_namer.md_idx_to_doc(context, opt_metadata)),
        ),
        ConfigContent::V1 {
            name,
            ty,
            encoded_bytes,
            decode_fn,
            opt_metadata,
            ..
        } => {
            let ty = ty.as_string(context);
            let bytes = encoded_bytes
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<String>>()
                .concat();
            Doc::line(
                Doc::text(format!(
                    "{} = config {}, {}, 0x{}",
                    name,
                    ty,
                    decode_fn.get().get_name(context),
                    bytes,
                ))
                .append(md_namer.md_idx_to_doc(context, opt_metadata)),
            )
        }
    }
}

fn function_to_doc<'a>(
    context: &'a Context,
    md_namer: &mut MetadataNamer,
    namer: &mut Namer,
    function: &'a FunctionContent,
    map_doc: &impl Fn(&Value, Doc) -> Doc,
) -> Doc {
    let public = if function.is_public { "pub " } else { "" };
    let entry = if function.is_entry { "entry " } else { "" };
    // TODO: Remove outer `if` once old encoding is fully removed.
    //       This is an intentional "complication" so that we see
    //       explicit using of `new_encoding` here.
    //       For the time being, for the old encoding, we don't want
    //       to show both `entry` and `entry_orig` although both
    //       values will be true.
    // TODO: When removing old encoding, remove also the TODO in the
    //       `rule fn_decl()` definition of the IR parser.
    let original_entry = if context.experimental.new_encoding {
        if function.is_original_entry {
            "entry_orig "
        } else {
            ""
        }
    } else if !function.is_entry && function.is_original_entry {
        "entry_orig "
    } else {
        ""
    };
    let fallback = if function.is_fallback {
        "fallback "
    } else {
        ""
    };
    Doc::line(
        Doc::text(format!(
            "{}{}{}{}fn {}",
            public, entry, original_entry, fallback, function.name
        ))
        .append(
            function
                .selector
                .map(|bytes| {
                    Doc::text(format!(
                        "<{:02x}{:02x}{:02x}{:02x}>",
                        bytes[0], bytes[1], bytes[2], bytes[3]
                    ))
                })
                .unwrap_or(Doc::Empty),
        )
        .append(Doc::in_parens_comma_sep(
            function
                .arguments
                .iter()
                .map(|(name, arg_val)| {
                    if let ValueContent {
                        value: ValueDatum::Argument(BlockArgument { ty, .. }),
                        metadata,
                        ..
                    } = &context.values[arg_val.0]
                    {
                        Doc::text(name)
                            .append(
                                Doc::Space.and(md_namer.md_idx_to_doc_no_comma(context, metadata)),
                            )
                            .append(Doc::text(format!(": {}", ty.as_string(context))))
                    } else {
                        unreachable!("Unexpected non argument value for function arguments.")
                    }
                })
                .collect(),
        ))
        .append(Doc::text(format!(
            " -> {}",
            function.return_type.as_string(context)
        )))
        .append(md_namer.md_idx_to_doc(context, &function.metadata))
        .append(Doc::text(" {")),
    )
    .append(Doc::indent(
        4,
        Doc::list_sep(
            vec![
                Doc::List(
                    function
                        .local_storage
                        .iter()
                        .map(|(name, var)| {
                            let var_content = &context.local_vars[var.0];
                            let init_doc = match &var_content.initializer {
                                Some(const_val) => Doc::text(format!(
                                    " = const {}",
                                    const_val.get_content(context).as_lit_string(context)
                                )),
                                None => Doc::Empty,
                            };
                            let mut_str = if var_content.mutable { "mut " } else { "" };
                            Doc::line(
                                // Print the inner, pointed-to type in the locals list.
                                Doc::text(format!(
                                    "local {mut_str}{} {name}",
                                    var.get_inner_type(context).as_string(context)
                                ))
                                .append(init_doc),
                            )
                        })
                        .collect(),
                ),
                Doc::list_sep(
                    function
                        .blocks
                        .iter()
                        .map(|block| block_to_doc(context, md_namer, namer, block, map_doc))
                        .collect(),
                    Doc::line(Doc::Empty),
                ),
            ],
            Doc::line(Doc::Empty),
        ),
    ))
    .append(Doc::text_line("}"))
}

fn block_to_doc(
    context: &Context,
    md_namer: &mut MetadataNamer,
    namer: &mut Namer,
    block: &Block,
    map_doc: &impl Fn(&Value, Doc) -> Doc,
) -> Doc {
    let block_content = &context.blocks[block.0];
    Doc::line(
        Doc::text(block_content.label.to_string()).append(
            Doc::in_parens_comma_sep(
                block
                    .arg_iter(context)
                    .map(|arg_val| {
                        Doc::text(namer.name(context, arg_val)).append(Doc::text(format!(
                            ": {}",
                            arg_val.get_type(context).unwrap().as_string(context)
                        )))
                    })
                    .collect(),
            )
            .append(Doc::Text(":".to_string())),
        ),
    )
    .append(Doc::List(
        block
            .instruction_iter(context)
            .map(|current_value| {
                let doc = instruction_to_doc(context, md_namer, namer, block, &current_value);
                (map_doc)(&current_value, doc)
            })
            .collect(),
    ))
}

fn constant_to_doc(
    context: &Context,
    md_namer: &mut MetadataNamer,
    namer: &mut Namer,
    const_val: &Value,
) -> Doc {
    if let ValueContent {
        value: ValueDatum::Constant(constant),
        metadata,
    } = &context.values[const_val.0]
    {
        Doc::line(
            Doc::text(format!(
                "{} = const {}",
                namer.name(context, const_val),
                constant.get_content(context).as_lit_string(context)
            ))
            .append(md_namer.md_idx_to_doc(context, metadata)),
        )
    } else {
        unreachable!("Not a constant value.")
    }
}

fn maybe_constant_to_doc(
    context: &Context,
    md_namer: &mut MetadataNamer,
    namer: &mut Namer,
    maybe_const_val: &Value,
) -> Doc {
    // Create a new doc only if value is new and unknown, and is a constant.
    if !namer.is_known(maybe_const_val) && maybe_const_val.is_constant(context) {
        constant_to_doc(context, md_namer, namer, maybe_const_val)
    } else {
        Doc::Empty
    }
}

fn instruction_to_doc<'a>(
    context: &'a Context,
    md_namer: &mut MetadataNamer,
    namer: &mut Namer,
    block: &Block,
    ins_value: &'a Value,
) -> Doc {
    if let ValueContent {
        value: ValueDatum::Instruction(instruction),
        metadata,
    } = &context.values[ins_value.0]
    {
        match &instruction.op {
            InstOp::AsmBlock(asm, args) => {
                asm_block_to_doc(context, md_namer, namer, ins_value, asm, args, metadata)
            }
            InstOp::BitCast(value, ty) => maybe_constant_to_doc(context, md_namer, namer, value)
                .append(Doc::line(
                    Doc::text(format!(
                        "{} = bitcast {} to {}",
                        namer.name(context, ins_value),
                        namer.name(context, value),
                        ty.as_string(context),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )),
            InstOp::UnaryOp { op, arg } => {
                let op_str = match op {
                    UnaryOpKind::Not => "not",
                };
                maybe_constant_to_doc(context, md_namer, namer, arg).append(Doc::line(
                    Doc::text(format!(
                        "{} = {op_str} {}",
                        namer.name(context, ins_value),
                        namer.name(context, arg),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                ))
            }
            InstOp::BinaryOp { op, arg1, arg2 } => {
                let op_str = match op {
                    BinaryOpKind::Add => "add",
                    BinaryOpKind::Sub => "sub",
                    BinaryOpKind::Mul => "mul",
                    BinaryOpKind::Div => "div",
                    BinaryOpKind::And => "and",
                    BinaryOpKind::Or => "or",
                    BinaryOpKind::Xor => "xor",
                    BinaryOpKind::Mod => "mod",
                    BinaryOpKind::Rsh => "rsh",
                    BinaryOpKind::Lsh => "lsh",
                };
                maybe_constant_to_doc(context, md_namer, namer, arg1)
                    .append(maybe_constant_to_doc(context, md_namer, namer, arg2))
                    .append(Doc::line(
                        Doc::text(format!(
                            "{} = {op_str} {}, {}",
                            namer.name(context, ins_value),
                            namer.name(context, arg1),
                            namer.name(context, arg2),
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    ))
            }
            InstOp::Branch(to_block) =>
            // Handle possibly constant block parameters
            {
                to_block
                    .args
                    .iter()
                    .fold(Doc::Empty, |doc, param| {
                        doc.append(maybe_constant_to_doc(context, md_namer, namer, param))
                    })
                    .append(Doc::line(
                        Doc::text(format!("br {}", context.blocks[to_block.block.0].label,))
                            .append(
                                Doc::in_parens_comma_sep(
                                    to_block
                                        .args
                                        .iter()
                                        .map(|arg_val| Doc::text(namer.name(context, arg_val)))
                                        .collect(),
                                )
                                .append(md_namer.md_idx_to_doc(context, metadata)),
                            ),
                    ))
            }
            InstOp::Call(func, args) => args
                .iter()
                .fold(Doc::Empty, |doc, arg_val| {
                    doc.append(maybe_constant_to_doc(context, md_namer, namer, arg_val))
                })
                .append(Doc::line(
                    Doc::text(format!(
                        "{} = call {}",
                        namer.name(context, ins_value),
                        context.functions[func.0].name
                    ))
                    .append(Doc::in_parens_comma_sep(
                        args.iter()
                            .map(|arg_val| Doc::text(namer.name(context, arg_val)))
                            .collect(),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )),
            InstOp::CastPtr(val, ty) => Doc::line(
                Doc::text(format!(
                    "{} = cast_ptr {} to {}",
                    namer.name(context, ins_value),
                    namer.name(context, val),
                    ty.as_string(context)
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            InstOp::Cmp(pred, lhs_value, rhs_value) => {
                let pred_str = match pred {
                    Predicate::Equal => "eq",
                    Predicate::LessThan => "lt",
                    Predicate::GreaterThan => "gt",
                };
                maybe_constant_to_doc(context, md_namer, namer, lhs_value)
                    .append(maybe_constant_to_doc(context, md_namer, namer, rhs_value))
                    .append(Doc::line(
                        Doc::text(format!(
                            "{} = cmp {pred_str} {} {}",
                            namer.name(context, ins_value),
                            namer.name(context, lhs_value),
                            namer.name(context, rhs_value),
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    ))
            }
            InstOp::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => {
                let true_label = &context.blocks[true_block.block.0].label;
                let false_label = &context.blocks[false_block.block.0].label;
                // Handle possibly constant block parameters
                let doc = true_block.args.iter().fold(
                    maybe_constant_to_doc(context, md_namer, namer, cond_value),
                    |doc, param| doc.append(maybe_constant_to_doc(context, md_namer, namer, param)),
                );
                let doc = false_block.args.iter().fold(doc, |doc, param| {
                    doc.append(maybe_constant_to_doc(context, md_namer, namer, param))
                });
                doc.append(Doc::line(
                    Doc::text(format!("cbr {}", namer.name(context, cond_value),)).append(
                        Doc::text(format!(", {true_label}")).append(
                            Doc::in_parens_comma_sep(
                                true_block
                                    .args
                                    .iter()
                                    .map(|arg_val| Doc::text(namer.name(context, arg_val)))
                                    .collect(),
                            )
                            .append(
                                Doc::text(format!(", {false_label}")).append(
                                    Doc::in_parens_comma_sep(
                                        false_block
                                            .args
                                            .iter()
                                            .map(|arg_val| Doc::text(namer.name(context, arg_val)))
                                            .collect(),
                                    )
                                    .append(md_namer.md_idx_to_doc(context, metadata)),
                                ),
                            ),
                        ),
                    ),
                ))
            }
            InstOp::ContractCall {
                return_type,
                name,
                params,
                coins,
                asset_id,
                gas,
            } => maybe_constant_to_doc(context, md_namer, namer, coins)
                .append(maybe_constant_to_doc(context, md_namer, namer, asset_id))
                .append(maybe_constant_to_doc(context, md_namer, namer, gas))
                .append(Doc::line(
                    Doc::text(format!(
                        "{} = contract_call {} {} {}, {}, {}, {}",
                        namer.name(context, ins_value),
                        return_type.as_string(context),
                        name.as_deref().unwrap_or(""),
                        namer.name(context, params),
                        namer.name(context, coins),
                        namer.name(context, asset_id),
                        namer.name(context, gas),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )),
            InstOp::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                FuelVmInstruction::Gtf { index, tx_field_id } => {
                    maybe_constant_to_doc(context, md_namer, namer, index).append(Doc::line(
                        Doc::text(format!(
                            "{} = gtf {}, {}",
                            namer.name(context, ins_value),
                            namer.name(context, index),
                            tx_field_id,
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    ))
                }
                FuelVmInstruction::Log {
                    log_val,
                    log_ty,
                    log_id,
                } => maybe_constant_to_doc(context, md_namer, namer, log_val)
                    .append(maybe_constant_to_doc(context, md_namer, namer, log_id))
                    .append(Doc::line(
                        Doc::text(format!(
                            "log {} {}, {}",
                            log_ty.as_string(context),
                            namer.name(context, log_val),
                            namer.name(context, log_id),
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    )),
                FuelVmInstruction::ReadRegister(reg) => Doc::line(
                    Doc::text(format!(
                        "{} = read_register {}",
                        namer.name(context, ins_value),
                        match reg {
                            Register::Of => "of",
                            Register::Pc => "pc",
                            Register::Ssp => "ssp",
                            Register::Sp => "sp",
                            Register::Fp => "fp",
                            Register::Hp => "hp",
                            Register::Error => "err",
                            Register::Ggas => "ggas",
                            Register::Cgas => "cgas",
                            Register::Bal => "bal",
                            Register::Is => "is",
                            Register::Ret => "ret",
                            Register::Retl => "retl",
                            Register::Flag => "flag",
                        },
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                ),
                FuelVmInstruction::Revert(v) => maybe_constant_to_doc(context, md_namer, namer, v)
                    .append(Doc::line(
                        Doc::text(format!("revert {}", namer.name(context, v),))
                            .append(md_namer.md_idx_to_doc(context, metadata)),
                    )),
                FuelVmInstruction::JmpMem => Doc::line(
                    Doc::text("jmp_mem".to_string())
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                ),
                FuelVmInstruction::Smo {
                    recipient,
                    message,
                    message_size,
                    coins,
                } => maybe_constant_to_doc(context, md_namer, namer, recipient)
                    .append(maybe_constant_to_doc(context, md_namer, namer, message))
                    .append(maybe_constant_to_doc(
                        context,
                        md_namer,
                        namer,
                        message_size,
                    ))
                    .append(maybe_constant_to_doc(context, md_namer, namer, coins))
                    .append(Doc::line(
                        Doc::text(format!(
                            "smo {}, {}, {}, {}",
                            namer.name(context, recipient),
                            namer.name(context, message),
                            namer.name(context, message_size),
                            namer.name(context, coins),
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    )),
                FuelVmInstruction::StateClear {
                    key,
                    number_of_slots,
                } => maybe_constant_to_doc(context, md_namer, namer, number_of_slots).append(
                    Doc::line(
                        Doc::text(format!(
                            "state_clear key {}, {}",
                            namer.name(context, key),
                            namer.name(context, number_of_slots),
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    ),
                ),
                FuelVmInstruction::StateLoadQuadWord {
                    load_val,
                    key,
                    number_of_slots,
                } => maybe_constant_to_doc(context, md_namer, namer, number_of_slots).append(
                    Doc::line(
                        Doc::text(format!(
                            "{} = state_load_quad_word {}, key {}, {}",
                            namer.name(context, ins_value),
                            namer.name(context, load_val),
                            namer.name(context, key),
                            namer.name(context, number_of_slots),
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    ),
                ),
                FuelVmInstruction::StateLoadWord(key) => Doc::line(
                    Doc::text(format!(
                        "{} = state_load_word key {}",
                        namer.name(context, ins_value),
                        namer.name(context, key),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                ),
                FuelVmInstruction::StateStoreQuadWord {
                    stored_val,
                    key,
                    number_of_slots,
                } => maybe_constant_to_doc(context, md_namer, namer, number_of_slots).append(
                    Doc::line(
                        Doc::text(format!(
                            "{} = state_store_quad_word {}, key {}, {}",
                            namer.name(context, ins_value),
                            namer.name(context, stored_val),
                            namer.name(context, key),
                            namer.name(context, number_of_slots),
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    ),
                ),
                FuelVmInstruction::StateStoreWord { stored_val, key } => {
                    maybe_constant_to_doc(context, md_namer, namer, stored_val).append(Doc::line(
                        Doc::text(format!(
                            "{} = state_store_word {}, key {}",
                            namer.name(context, ins_value),
                            namer.name(context, stored_val),
                            namer.name(context, key),
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    ))
                }
                FuelVmInstruction::WideUnaryOp { op, arg, result } => {
                    let op_str = match op {
                        UnaryOpKind::Not => "not",
                    };
                    maybe_constant_to_doc(context, md_namer, namer, arg).append(Doc::line(
                        Doc::text(format!(
                            "wide {op_str} {} to {}",
                            namer.name(context, arg),
                            namer.name(context, result),
                        ))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                    ))
                }
                FuelVmInstruction::WideBinaryOp {
                    op,
                    arg1,
                    arg2,
                    result,
                } => {
                    let op_str = match op {
                        BinaryOpKind::Add => "add",
                        BinaryOpKind::Sub => "sub",
                        BinaryOpKind::Mul => "mul",
                        BinaryOpKind::Div => "div",
                        BinaryOpKind::And => "and",
                        BinaryOpKind::Or => "or",
                        BinaryOpKind::Xor => "xor",
                        BinaryOpKind::Mod => "mod",
                        BinaryOpKind::Rsh => "rsh",
                        BinaryOpKind::Lsh => "lsh",
                    };
                    maybe_constant_to_doc(context, md_namer, namer, arg1)
                        .append(maybe_constant_to_doc(context, md_namer, namer, arg2))
                        .append(Doc::line(
                            Doc::text(format!(
                                "wide {op_str} {}, {} to {}",
                                namer.name(context, arg1),
                                namer.name(context, arg2),
                                namer.name(context, result),
                            ))
                            .append(md_namer.md_idx_to_doc(context, metadata)),
                        ))
                }
                FuelVmInstruction::WideModularOp {
                    op,
                    result,
                    arg1,
                    arg2,
                    arg3,
                } => {
                    let op_str = match op {
                        BinaryOpKind::Mod => "mod",
                        _ => unreachable!(),
                    };
                    maybe_constant_to_doc(context, md_namer, namer, arg1)
                        .append(maybe_constant_to_doc(context, md_namer, namer, arg2))
                        .append(maybe_constant_to_doc(context, md_namer, namer, arg3))
                        .append(Doc::line(
                            Doc::text(format!(
                                "wide {op_str} {}, {}, {} to {}",
                                namer.name(context, arg1),
                                namer.name(context, arg2),
                                namer.name(context, arg3),
                                namer.name(context, result),
                            ))
                            .append(md_namer.md_idx_to_doc(context, metadata)),
                        ))
                }
                FuelVmInstruction::WideCmpOp { op, arg1, arg2 } => {
                    let pred_str = match op {
                        Predicate::Equal => "eq",
                        Predicate::LessThan => "lt",
                        Predicate::GreaterThan => "gt",
                    };
                    maybe_constant_to_doc(context, md_namer, namer, arg1)
                        .append(maybe_constant_to_doc(context, md_namer, namer, arg2))
                        .append(Doc::line(
                            Doc::text(format!(
                                "{} = wide cmp {pred_str} {} {}",
                                namer.name(context, ins_value),
                                namer.name(context, arg1),
                                namer.name(context, arg2),
                            ))
                            .append(md_namer.md_idx_to_doc(context, metadata)),
                        ))
                }
                FuelVmInstruction::Retd { ptr, len } => {
                    maybe_constant_to_doc(context, md_namer, namer, ptr)
                        .append(maybe_constant_to_doc(context, md_namer, namer, len))
                        .append(Doc::line(
                            Doc::text(format!(
                                "retd {} {}",
                                namer.name(context, ptr),
                                namer.name(context, len),
                            ))
                            .append(md_namer.md_idx_to_doc(context, metadata)),
                        ))
                }
            },
            InstOp::GetElemPtr {
                base,
                elem_ptr_ty,
                indices,
            } => indices
                .iter()
                .fold(Doc::Empty, |acc, idx| {
                    acc.append(maybe_constant_to_doc(context, md_namer, namer, idx))
                })
                .append(Doc::line(
                    Doc::text(format!(
                        "{} = get_elem_ptr {}, {}, ",
                        namer.name(context, ins_value),
                        namer.name(context, base),
                        elem_ptr_ty.as_string(context),
                    ))
                    .append(Doc::list_sep(
                        indices
                            .iter()
                            .map(|idx| Doc::text(namer.name(context, idx)))
                            .collect(),
                        Doc::Comma,
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )),
            InstOp::GetLocal(local_var) => {
                let name = block
                    .get_function(context)
                    .lookup_local_name(context, local_var)
                    .unwrap();
                Doc::line(
                    Doc::text(format!(
                        "{} = get_local {}, {name}",
                        namer.name(context, ins_value),
                        local_var.get_type(context).as_string(context),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )
            }
            InstOp::GetGlobal(global_var) => {
                let name = block
                    .get_function(context)
                    .get_module(context)
                    .lookup_global_variable_name(context, global_var)
                    .unwrap();
                Doc::line(
                    Doc::text(format!(
                        "{} = get_global {}, {name}",
                        namer.name(context, ins_value),
                        global_var.get_type(context).as_string(context),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )
            }
            InstOp::GetConfig(_, name) => Doc::line(
                match block.get_module(context).get_config(context, name).unwrap() {
                    ConfigContent::V0 { name, ptr_ty, .. }
                    | ConfigContent::V1 { name, ptr_ty, .. } => Doc::text(format!(
                        "{} = get_config {}, {}",
                        namer.name(context, ins_value),
                        ptr_ty.as_string(context),
                        name,
                    )),
                }
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            InstOp::GetStorageKey(storage_key) => {
                let name = block
                    .get_function(context)
                    .get_module(context)
                    .lookup_storage_key_path(context, storage_key)
                    .unwrap();
                Doc::line(
                    Doc::text(format!(
                        "{} = get_storage_key {}, {name}",
                        namer.name(context, ins_value),
                        storage_key.get_type(context).as_string(context),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )
            }
            InstOp::IntToPtr(value, ty) => maybe_constant_to_doc(context, md_namer, namer, value)
                .append(Doc::line(
                    Doc::text(format!(
                        "{} = int_to_ptr {} to {}",
                        namer.name(context, ins_value),
                        namer.name(context, value),
                        ty.as_string(context),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )),
            InstOp::Load(src_value) => Doc::line(
                Doc::text(format!(
                    "{} = load {}",
                    namer.name(context, ins_value),
                    namer.name(context, src_value),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            InstOp::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                byte_len,
            } => Doc::line(
                Doc::text(format!(
                    "mem_copy_bytes {}, {}, {}",
                    namer.name(context, dst_val_ptr),
                    namer.name(context, src_val_ptr),
                    byte_len,
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } => Doc::line(
                Doc::text(format!(
                    "mem_copy_val {}, {}",
                    namer.name(context, dst_val_ptr),
                    namer.name(context, src_val_ptr),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            InstOp::MemClearVal { dst_val_ptr } => Doc::line(
                Doc::text(format!(
                    "mem_clear_val {}",
                    namer.name(context, dst_val_ptr),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            InstOp::Nop => Doc::line(
                Doc::text(format!("{} = nop", namer.name(context, ins_value)))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            InstOp::PtrToInt(value, ty) => maybe_constant_to_doc(context, md_namer, namer, value)
                .append(Doc::line(
                    Doc::text(format!(
                        "{} = ptr_to_int {} to {}",
                        namer.name(context, ins_value),
                        namer.name(context, value),
                        ty.as_string(context),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )),
            InstOp::Ret(v, t) => {
                maybe_constant_to_doc(context, md_namer, namer, v).append(Doc::line(
                    Doc::text(format!(
                        "ret {} {}",
                        t.as_string(context),
                        namer.name(context, v),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                ))
            }
            InstOp::Store {
                dst_val_ptr,
                stored_val,
            } => maybe_constant_to_doc(context, md_namer, namer, stored_val).append(Doc::line(
                Doc::text(format!(
                    "store {} to {}",
                    namer.name(context, stored_val),
                    namer.name(context, dst_val_ptr),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            )),
        }
    } else {
        unreachable!("Unexpected non instruction for block contents.")
    }
}

fn asm_block_to_doc(
    context: &Context,
    md_namer: &mut MetadataNamer,
    namer: &mut Namer,
    ins_value: &Value,
    asm: &AsmBlock,
    args: &[AsmArg],
    metadata: &Option<MetadataIndex>,
) -> Doc {
    let AsmBlock {
        body,
        return_type,
        return_name,
        ..
    } = &asm;
    args.iter()
        .fold(
            Doc::Empty,
            |doc, AsmArg { initializer, .. }| match initializer {
                Some(init_val) if init_val.is_constant(context) => {
                    doc.append(maybe_constant_to_doc(context, md_namer, namer, init_val))
                }
                _otherwise => doc,
            },
        )
        .append(Doc::line(
            Doc::text(format!("{} = asm", namer.name(context, ins_value)))
                .append(Doc::in_parens_comma_sep(
                    args.iter()
                        .map(|AsmArg { name, initializer }| {
                            Doc::text(name.as_str()).append(match initializer {
                                Some(init_val) => {
                                    Doc::text(format!(": {}", namer.name(context, init_val)))
                                }
                                None => Doc::Empty,
                            })
                        })
                        .collect(),
                ))
                .append(
                    Doc::text(format!(
                        " -> {}{}",
                        return_type.as_string(context),
                        return_name
                            .as_ref()
                            .map_or("".to_string(), |rn| format!(" {rn}"))
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )
                .append(Doc::text(" {")),
        ))
        .append(Doc::indent(
            4,
            Doc::List(
                body.iter()
                    .map(
                        |AsmInstruction {
                             op_name: name,
                             args,
                             immediate,
                             metadata,
                         }| {
                            Doc::line(
                                Doc::text(format!("{:6} ", name.as_str())).append(
                                    Doc::list_sep(
                                        args.iter().map(|arg| Doc::text(arg.as_str())).collect(),
                                        Doc::text(" "),
                                    )
                                    .append(match immediate {
                                        Some(imm_str) => Doc::text(format!(" {imm_str}")),
                                        None => Doc::Empty,
                                    })
                                    .append(md_namer.md_idx_to_doc(context, metadata)),
                                ),
                            )
                        },
                    )
                    .collect(),
            ),
        ))
        .append(Doc::text_line("}"))
}

impl ConstantContent {
    fn as_lit_string(&self, context: &Context) -> String {
        match &self.value {
            ConstantValue::Undef => format!("{} undef", self.ty.as_string(context)),
            ConstantValue::Unit => "unit ()".into(),
            ConstantValue::Bool(b) => format!("bool {}", if *b { "true" } else { "false" }),
            ConstantValue::Uint(v) => format!("{} {}", self.ty.as_string(context), v),
            ConstantValue::U256(v) => {
                let bytes = v.to_be_bytes();
                format!(
                    "u256 0x{}",
                    bytes
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<Vec<String>>()
                        .concat()
                )
            }
            ConstantValue::B256(v) => {
                let bytes = v.to_be_bytes();
                format!(
                    "b256 0x{}",
                    bytes
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<Vec<String>>()
                        .concat()
                )
            }
            ConstantValue::String(bs) => format!(
                "{} \"{}\"",
                self.ty.as_string(context),
                bs.iter()
                    .map(
                        |b| if b.is_ascii() && !b.is_ascii_control() && *b != b'\\' && *b != b'"' {
                            format!("{}", *b as char)
                        } else {
                            format!("\\x{b:02x}")
                        }
                    )
                    .collect::<Vec<_>>()
                    .join("")
            ),
            ConstantValue::Array(elems) => format!(
                "{} [{}]",
                self.ty.as_string(context),
                elems
                    .iter()
                    .map(|elem| elem.as_lit_string(context))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ConstantValue::Slice(elems) => format!(
                "__slice[{}] [{}]",
                self.ty.as_string(context),
                elems
                    .iter()
                    .map(|elem| elem.as_lit_string(context))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ConstantValue::Struct(fields) => format!(
                "{} {{ {} }}",
                self.ty.as_string(context),
                fields
                    .iter()
                    .map(|field| field.as_lit_string(context))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ConstantValue::Reference(constant) => format!("&({})", constant.as_lit_string(context)),
            ConstantValue::RawUntypedSlice(bytes) => {
                format!(
                    "{} 0x{}",
                    self.ty.as_string(context),
                    bytes
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<Vec<String>>()
                        .concat()
                )
            }
        }
    }
}

struct Namer {
    function: Function,
    names: HashMap<Value, String>,
    next_value_idx: u64,
}

impl Namer {
    fn new(function: Function) -> Self {
        Namer {
            function,
            names: HashMap::new(),
            next_value_idx: 0,
        }
    }

    fn name(&mut self, context: &Context, value: &Value) -> String {
        match &context.values[value.0].value {
            ValueDatum::Argument(_) => self
                .function
                .lookup_arg_name(context, value)
                .cloned()
                .unwrap_or_else(|| self.default_name(value)),
            ValueDatum::Constant(_) => self.default_name(value),
            ValueDatum::Instruction(_) => self.default_name(value),
        }
    }

    fn default_name(&mut self, value: &Value) -> String {
        self.names.get(value).cloned().unwrap_or_else(|| {
            let new_name = format!("v{}", self.next_value_idx);
            self.next_value_idx += 1;
            self.names.insert(*value, new_name.clone());
            new_name
        })
    }

    fn is_known(&self, value: &Value) -> bool {
        self.names.contains_key(value)
    }
}

#[derive(Default)]
struct MetadataNamer {
    md_map: BTreeMap<MetadataIndex, u64>,
    next_md_idx: u64,
}

impl MetadataNamer {
    fn values_sorted(&self) -> impl Iterator<Item = (u64, MetadataIndex)> {
        let mut items = self
            .md_map
            .clone()
            .into_iter()
            .map(|(a, b)| (b, a))
            .collect::<Vec<_>>();
        items.sort_unstable();
        items.into_iter()
    }

    fn get(&self, md_idx: &MetadataIndex) -> Option<u64> {
        self.md_map.get(md_idx).copied()
    }

    // This method is how we introduce 'valid' metadata to the namer, as only valid metadata are
    // printed at the end.  Since metadata are stored globally to the context there may be a bunch
    // in there which aren't relevant (e.g., library code).  Hopefully this will go away when the
    // Sway compiler becomes properly modular and eschews all the inlining it does.
    //
    // So, we insert a reference index into the namer whenever we see a new metadata index passed
    // here.  But we also need to recursively 'validate' any other metadata referred to, e.g., list
    // elements, struct members, etc. It's done in `add_md_idx()` below.
    fn md_idx_to_doc_no_comma(&mut self, context: &Context, md_idx: &Option<MetadataIndex>) -> Doc {
        md_idx
            .map(|md_idx| Doc::text(format!("!{}", self.add_md_idx(context, &md_idx))))
            .unwrap_or(Doc::Empty)
    }

    fn md_idx_to_doc(&mut self, context: &Context, md_idx: &Option<MetadataIndex>) -> Doc {
        Doc::Comma.and(self.md_idx_to_doc_no_comma(context, md_idx))
    }

    fn add_md_idx(&mut self, context: &Context, md_idx: &MetadataIndex) -> u64 {
        self.md_map.get(md_idx).copied().unwrap_or_else(|| {
            // Recurse for all sub-metadata here first to be sure they can be referenced later.
            self.add_md(context, &context.metadata[md_idx.0]);

            // Create a new index mapping.
            let new_idx = self.next_md_idx;
            self.next_md_idx += 1;
            self.md_map.insert(*md_idx, new_idx);
            new_idx
        })
    }

    fn add_md(&mut self, context: &Context, md: &Metadatum) {
        match md {
            Metadatum::Integer(_) | Metadatum::String(_) | Metadatum::SourceId(_) => (),
            Metadatum::Index(idx) => {
                let _ = self.add_md_idx(context, idx);
            }
            Metadatum::Struct(_tag, els) => {
                for el in els {
                    self.add_md(context, el);
                }
            }
            Metadatum::List(idcs) => {
                for idx in idcs {
                    self.add_md_idx(context, idx);
                }
            }
        }
    }

    fn to_doc(&self, context: &Context) -> Doc {
        fn md_to_string(
            md_namer: &MetadataNamer,
            md: &Metadatum,
            source_engine: &SourceEngine,
        ) -> String {
            match md {
                Metadatum::Integer(i) => i.to_string(),
                Metadatum::Index(idx) => format!(
                    "!{}",
                    md_namer
                        .get(idx)
                        .unwrap_or_else(|| panic!("Metadata index ({idx:?}) not found in namer."))
                ),
                Metadatum::String(s) => format!("{s:?}"),
                Metadatum::SourceId(id) => {
                    let path = source_engine.get_path(id);
                    format!("{path:?}")
                }
                Metadatum::Struct(tag, els) => {
                    format!(
                        "{tag} {}",
                        els.iter()
                            .map(|el_md| md_to_string(md_namer, el_md, source_engine))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                }
                Metadatum::List(idcs) => {
                    format!(
                        "({})",
                        idcs.iter()
                            .map(|idx| format!(
                                "!{}",
                                md_namer.get(idx).unwrap_or_else(|| panic!(
                                    "Metadata list index ({idx:?}) not found in namer."
                                ))
                            ))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                }
            }
        }

        let md_lines = self
            .values_sorted()
            .map(|(ref_idx, md_idx)| {
                Doc::text_line(format!(
                    "!{ref_idx} = {}",
                    md_to_string(self, &context.metadata[md_idx.0], context.source_engine)
                ))
            })
            .collect::<Vec<_>>();

        // We want to add an empty line only when there are metadata.
        if md_lines.is_empty() {
            Doc::Empty
        } else {
            Doc::line(Doc::Empty).append(Doc::List(md_lines))
        }
    }
}

/// There will be a much more efficient way to do this, but for now this will do.
fn build_doc(doc: Doc, indent: i64) -> String {
    match doc {
        Doc::Empty => "".into(),
        Doc::Space => " ".into(),
        Doc::Comma => ", ".into(),

        Doc::Text(t) => t,
        Doc::Line(d) => {
            if matches!(*d, Doc::Empty) {
                "\n".into()
            } else {
                format!("{}{}\n", " ".repeat(indent as usize), build_doc(*d, indent))
            }
        }

        Doc::Pair(l, r) => [build_doc(*l, indent), build_doc(*r, indent)].concat(),

        Doc::List(v) => v
            .into_iter()
            .map(|d| build_doc(d, indent))
            .collect::<Vec<String>>()
            .concat(),
        Doc::ListSep(v, s) => v
            .into_iter()
            .filter_map(|d| match &d {
                Doc::Empty => None,
                Doc::List(vs) => {
                    if vs.is_empty() {
                        None
                    } else {
                        Some(build_doc(d, indent))
                    }
                }
                _ => Some(build_doc(d, indent)),
            })
            .collect::<Vec<String>>()
            .join(&build_doc(*s, indent)),

        Doc::Parens(d) => format!("({})", build_doc(*d, indent)),

        Doc::Indent(n, d) => build_doc(*d, indent + n),
    }
}
