//! Print (or serialize) IR to human and machine readable text.
//!
//! This module implements a document based pretty-printer.  A couple of 3rd party pretty printing
//! crates were assessed but didn't seem to work as well as this simple version, which is quite
//! effective.

use std::collections::{BTreeMap, HashMap};

use crate::{
    asm::*,
    block::Block,
    constant::{Constant, ConstantValue},
    context::Context,
    function::{Function, FunctionContent},
    instruction::{Instruction, Predicate, Register},
    irtype::Type,
    metadata::{MetadataIndex, Metadatum},
    module::{Kind, ModuleContent},
    value::{Value, ValueContent, ValueDatum},
    BinaryOpKind, BlockArgument,
};

#[derive(Debug)]
enum Doc {
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
    fn text<S: Into<String>>(s: S) -> Self {
        Doc::Text(s.into())
    }

    fn line(doc: Doc) -> Self {
        Doc::Line(Box::new(doc))
    }

    fn text_line<S: Into<String>>(s: S) -> Self {
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

    fn append(self, doc: Doc) -> Doc {
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

    fn build(self) -> String {
        build_doc(self, 0)
    }
}

/// Pretty-print a whole [`Context`] to a string.
///
/// The ouput from this function must always be suitable for [`crate::parser::parse`].
pub fn to_string(context: &Context) -> String {
    let mut md_namer = MetadataNamer::default();
    context
        .modules
        .iter()
        .fold(Doc::Empty, |doc, (_, module)| {
            doc.append(module_to_doc(context, &mut md_namer, module))
        })
        .append(md_namer.to_doc(context))
        .build()
}

fn module_to_doc<'a>(
    context: &'a Context,
    md_namer: &mut MetadataNamer,
    module: &'a ModuleContent,
) -> Doc {
    Doc::line(Doc::Text(format!(
        "{} {{",
        match module.kind {
            Kind::Contract => "contract",
            Kind::Library => "library",
            Kind::Predicate => "predicate ",
            Kind::Script => "script",
        }
    )))
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
                    )
                })
                .collect(),
            Doc::line(Doc::Empty),
        ),
    ))
    .append(Doc::text_line("}"))
}

fn function_to_doc<'a>(
    context: &'a Context,
    md_namer: &mut MetadataNamer,
    namer: &mut Namer,
    function: &'a FunctionContent,
) -> Doc {
    Doc::line(
        Doc::text(format!("fn {}", function.name,))
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
                                    Doc::Space
                                        .and(md_namer.md_idx_to_doc_no_comma(context, metadata)),
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
                        .map(|(name, ptr)| {
                            let ptr_content = &context.pointers[ptr.0];
                            let init_doc = match &ptr_content.initializer {
                                Some(const_val) => Doc::text(format!(
                                    " = const {}",
                                    const_val.as_lit_string(context)
                                )),
                                None => Doc::Empty,
                            };
                            Doc::line(
                                Doc::text(format!("local {}", ptr.as_string(context, Some(name))))
                                    .append(init_doc),
                            )
                        })
                        .collect(),
                ),
                Doc::list_sep(
                    function
                        .blocks
                        .iter()
                        .map(|block| block_to_doc(context, md_namer, namer, block))
                        .collect(),
                    Doc::line(Doc::Empty),
                ),
            ],
            Doc::line(Doc::Empty),
        ),
    ))
    .append(Doc::text_line("}"))
}

fn block_to_doc<'a>(
    context: &'a Context,
    md_namer: &mut MetadataNamer,
    namer: &mut Namer,
    block: &Block,
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
        block_content
            .instructions
            .iter()
            .map(|ins| instruction_to_doc(context, md_namer, namer, block, ins))
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
                constant.as_lit_string(context)
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
        match instruction {
            Instruction::AsmBlock(asm, args) => {
                asm_block_to_doc(context, md_namer, namer, ins_value, asm, args, metadata)
            }
            Instruction::AddrOf(value) => maybe_constant_to_doc(context, md_namer, namer, value)
                .append(
                    Doc::text_line(format!(
                        "{} = addr_of {}",
                        namer.name(context, ins_value),
                        namer.name(context, value),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                ),
            Instruction::BitCast(value, ty) => {
                maybe_constant_to_doc(context, md_namer, namer, value).append(Doc::line(
                    Doc::text(format!(
                        "{} = bitcast {} to {}",
                        namer.name(context, ins_value),
                        namer.name(context, value),
                        ty.as_string(context),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                ))
            }
            Instruction::BinaryOp { op, arg1, arg2 } => {
                let op_str = match op {
                    BinaryOpKind::Add => "add",
                    BinaryOpKind::Sub => "sub",
                    BinaryOpKind::Mul => "mul",
                    BinaryOpKind::Div => "div",
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
            Instruction::Branch(to_block) =>
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
            Instruction::Call(func, args) => args
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
            Instruction::Cmp(pred, lhs_value, rhs_value) => {
                let pred_str = match pred {
                    Predicate::Equal => "eq",
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
            Instruction::ConditionalBranch {
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
            Instruction::ContractCall {
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
                        name,
                        namer.name(context, params),
                        namer.name(context, coins),
                        namer.name(context, asset_id),
                        namer.name(context, gas),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )),
            Instruction::ExtractElement {
                array,
                ty,
                index_val,
            } => maybe_constant_to_doc(context, md_namer, namer, index_val).append(Doc::line(
                Doc::text(format!(
                    "{} = extract_element {}, {}, {}",
                    namer.name(context, ins_value),
                    namer.name(context, array),
                    Type::Array(*ty).as_string(context),
                    namer.name(context, index_val),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            )),
            Instruction::ExtractValue {
                aggregate,
                ty,
                indices,
            } => maybe_constant_to_doc(context, md_namer, namer, aggregate).append(Doc::line(
                Doc::text(format!(
                    "{} = extract_value {}, {}, ",
                    namer.name(context, ins_value),
                    namer.name(context, aggregate),
                    Type::Struct(*ty).as_string(context),
                ))
                .append(Doc::list_sep(
                    indices
                        .iter()
                        .map(|idx| Doc::text(format!("{idx}")))
                        .collect(),
                    Doc::Comma,
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            )),
            Instruction::GetStorageKey => Doc::line(
                Doc::text(format!(
                    "{} = get_storage_key",
                    namer.name(context, ins_value),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            Instruction::Gtf { index, tx_field_id } => {
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
            Instruction::GetPointer {
                base_ptr,
                ptr_ty,
                offset,
            } => {
                let name = block
                    .get_function(context)
                    .lookup_local_name(context, base_ptr)
                    .unwrap();
                Doc::line(
                    Doc::text(format!(
                        "{} = get_ptr {}, {}, {}",
                        namer.name(context, ins_value),
                        base_ptr.as_string(context, Some(name)),
                        ptr_ty.as_string(context, None),
                        offset,
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )
            }
            Instruction::InsertElement {
                array,
                ty,
                value,
                index_val,
            } => maybe_constant_to_doc(context, md_namer, namer, array)
                .append(maybe_constant_to_doc(context, md_namer, namer, value))
                .append(maybe_constant_to_doc(context, md_namer, namer, index_val))
                .append(Doc::line(
                    Doc::text(format!(
                        "{} = insert_element {}, {}, {}, {}",
                        namer.name(context, ins_value),
                        namer.name(context, array),
                        Type::Array(*ty).as_string(context),
                        namer.name(context, value),
                        namer.name(context, index_val),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )),
            Instruction::InsertValue {
                aggregate,
                ty,
                value,
                indices,
            } => maybe_constant_to_doc(context, md_namer, namer, aggregate)
                .append(maybe_constant_to_doc(context, md_namer, namer, value))
                .append(Doc::line(
                    Doc::text(format!(
                        "{} = insert_value {}, {}, {}, ",
                        namer.name(context, ins_value),
                        namer.name(context, aggregate),
                        Type::Struct(*ty).as_string(context),
                        namer.name(context, value),
                    ))
                    .append(Doc::list_sep(
                        indices
                            .iter()
                            .map(|idx| Doc::text(format!("{idx}")))
                            .collect(),
                        Doc::Comma,
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                )),
            Instruction::IntToPtr(value, ty) => {
                maybe_constant_to_doc(context, md_namer, namer, value).append(Doc::line(
                    Doc::text(format!(
                        "{} = int_to_ptr {} to {}",
                        namer.name(context, ins_value),
                        namer.name(context, value),
                        ty.as_string(context),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                ))
            }
            Instruction::Load(src_value) => Doc::line(
                Doc::text(format!(
                    "{} = load ptr {}",
                    namer.name(context, ins_value),
                    namer.name(context, src_value),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            Instruction::Log {
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
            Instruction::MemCopy {
                dst_val,
                src_val,
                byte_len,
            } => maybe_constant_to_doc(context, md_namer, namer, src_val).append(Doc::line(
                Doc::text(format!(
                    "mem_copy {}, {}, {}",
                    namer.name(context, dst_val),
                    namer.name(context, src_val),
                    byte_len,
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            )),
            Instruction::Nop => Doc::line(
                Doc::text(format!("{} = nop", namer.name(context, ins_value)))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            Instruction::ReadRegister(reg) => Doc::line(
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
            Instruction::Ret(v, t) => {
                maybe_constant_to_doc(context, md_namer, namer, v).append(Doc::line(
                    Doc::text(format!(
                        "ret {} {}",
                        t.as_string(context),
                        namer.name(context, v),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                ))
            }
            Instruction::Revert(v) => {
                maybe_constant_to_doc(context, md_namer, namer, v).append(Doc::line(
                    Doc::text(format!("revert {}", namer.name(context, v),))
                        .append(md_namer.md_idx_to_doc(context, metadata)),
                ))
            }
            Instruction::StateLoadQuadWord { load_val, key } => Doc::line(
                Doc::text(format!(
                    "state_load_quad_word ptr {}, key ptr {}",
                    namer.name(context, load_val),
                    namer.name(context, key),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            Instruction::StateLoadWord(key) => Doc::line(
                Doc::text(format!(
                    "{} = state_load_word key ptr {}",
                    namer.name(context, ins_value),
                    namer.name(context, key),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            Instruction::StateStoreQuadWord { stored_val, key } => Doc::line(
                Doc::text(format!(
                    "state_store_quad_word ptr {}, key ptr {}",
                    namer.name(context, stored_val),
                    namer.name(context, key),
                ))
                .append(md_namer.md_idx_to_doc(context, metadata)),
            ),
            Instruction::StateStoreWord { stored_val, key } => {
                maybe_constant_to_doc(context, md_namer, namer, stored_val).append(Doc::line(
                    Doc::text(format!(
                        "state_store_word {}, key ptr {}",
                        namer.name(context, stored_val),
                        namer.name(context, key),
                    ))
                    .append(md_namer.md_idx_to_doc(context, metadata)),
                ))
            }
            Instruction::Store {
                dst_val,
                stored_val,
            } => maybe_constant_to_doc(context, md_namer, namer, stored_val).append(Doc::line(
                Doc::text(format!(
                    "store {}, ptr {}",
                    namer.name(context, stored_val),
                    namer.name(context, dst_val),
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
    let AsmBlockContent {
        body,
        return_type,
        return_name,
        ..
    } = &context.asm_blocks[asm.0];
    args.iter()
        .fold(
            Doc::Empty,
            |doc, AsmArg { initializer, .. }| match initializer {
                Some(init_val) if init_val.is_constant(context) => {
                    doc.append(constant_to_doc(context, md_namer, namer, init_val))
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
                    return_name
                        .as_ref()
                        .map(|rn| {
                            Doc::text(format!(" -> {} {rn}", return_type.as_string(context)))
                                .append(md_namer.md_idx_to_doc(context, metadata))
                        })
                        .unwrap_or(Doc::Empty),
                )
                .append(Doc::text(" {")),
        ))
        .append(Doc::indent(
            4,
            Doc::List(
                body.iter()
                    .map(
                        |AsmInstruction {
                             name,
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

impl Constant {
    fn as_lit_string(&self, context: &Context) -> String {
        match &self.value {
            ConstantValue::Undef => format!("{} undef", self.ty.as_string(context)),
            ConstantValue::Unit => "unit ()".into(),
            ConstantValue::Bool(b) => format!("bool {}", if *b { "true" } else { "false" }),
            ConstantValue::Uint(v) => format!("{} {}", self.ty.as_string(context), v),
            ConstantValue::B256(bs) => format!(
                "b256 0x{}",
                bs.iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<Vec<String>>()
                    .concat()
            ),
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
            ConstantValue::Struct(fields) => format!(
                "{} {{ {} }}",
                self.ty.as_string(context),
                fields
                    .iter()
                    .map(|field| field.as_lit_string(context))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

struct Namer {
    function: Function,

    names:          HashMap<Value, String>,
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
    md_map:      BTreeMap<MetadataIndex, u64>,
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
            Metadatum::Integer(_) | Metadatum::String(_) => (),
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
        fn md_to_string(context: &Context, md_namer: &MetadataNamer, md: &Metadatum) -> String {
            match md {
                Metadatum::Integer(i) => i.to_string(),
                Metadatum::Index(idx) => format!(
                    "!{}",
                    md_namer
                        .get(idx)
                        .unwrap_or_else(|| panic!("Metadata index ({idx:?}) not found in namer."))
                ),
                Metadatum::String(s) => format!("{s:?}"),
                Metadatum::Struct(tag, els) => {
                    format!(
                        "{tag} {}",
                        els.iter()
                            .map(|el_md| md_to_string(context, md_namer, el_md))
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
                    md_to_string(context, self, &context.metadata[md_idx.0])
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
