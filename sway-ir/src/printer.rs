//! Print (or serialize) IR to human and machine readable text.
//!
//! This module implements a document based pretty-printer.  A couple of 3rd party pretty printing
//! crates were assessed but didn't seem to work as well as this simple version, which is quite
//! effective.

use crate::{
    asm::*,
    block::Block,
    constant::{Constant, ConstantValue},
    context::Context,
    function::{Function, FunctionContent},
    instruction::Instruction,
    irtype::Type,
    module::{Kind, ModuleContent},
    pointer::{Pointer, PointerContent},
    value::{Value, ValueContent},
};

#[derive(Debug)]
enum Doc {
    Empty,
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
        match self {
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
    context
        .modules
        .iter()
        .fold(Doc::Empty, |doc, (_, module)| {
            doc.append(module_to_doc(context, module))
        })
        .build()
}

fn module_to_doc<'a>(context: &'a Context, module: &'a ModuleContent) -> Doc {
    Doc::line(Doc::Text(format!(
        "{} {} {{",
        match module.kind {
            Kind::Contract => "contract",
            Kind::Library => "library",
            Kind::Predicate => "predicate ",
            Kind::Script => "script",
        },
        &module.name
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
                        &mut Namer::new(*function),
                        &context.functions[function.0],
                    )
                })
                .collect(),
            Doc::line(Doc::Empty),
        ),
    ))
    .append(Doc::line(Doc::text("}")))
}

fn function_to_doc<'a>(
    context: &'a Context,
    namer: &mut Namer,
    function: &'a FunctionContent,
) -> Doc {
    Doc::line(
        Doc::text(format!(
            "{}fn {}{}",
            if function.is_public { "pub " } else { "" },
            function.name,
            match function.selector {
                None => "".to_owned(),
                Some(bytes) => format!(
                    "<{:02x}{:02x}{:02x}{:02x}>",
                    bytes[0], bytes[1], bytes[2], bytes[3]
                ),
            }
        ))
        .append(Doc::in_parens_comma_sep(
            function
                .arguments
                .iter()
                .map(|(name, arg_val)| {
                    let ty = match &context.values[arg_val.0] {
                        ValueContent::Argument(ty) => ty,
                        _ => unreachable!("Unexpected non argument value for function arguments."),
                    };
                    Doc::text(format!("{}: {}", name, ty.as_string(context),))
                })
                .collect(),
        ))
        .append(Doc::text(format!(
            " -> {} {{",
            function.return_type.as_string(context)
        ))),
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
                                Doc::text(format!("local {}", ptr.as_string(context, name)))
                                    .append(init_doc),
                            )
                        })
                        .collect(),
                ),
                Doc::list_sep(
                    function
                        .blocks
                        .iter()
                        .map(|block| block_to_doc(context, namer, block))
                        .collect(),
                    Doc::line(Doc::Empty),
                ),
            ],
            Doc::line(Doc::Empty),
        ),
    ))
    .append(Doc::line(Doc::text("}")))
}

fn block_to_doc<'a>(context: &'a Context, namer: &mut Namer, block: &Block) -> Doc {
    let block_content = &context.blocks[block.0];
    Doc::line(Doc::text(format!("{}:", block_content.label))).append(Doc::List(
        block_content
            .instructions
            .iter()
            .map(|ins| instruction_to_doc(context, namer, block, ins))
            .collect(),
    ))
}

fn constant_to_doc(context: &Context, namer: &mut Namer, const_val: &Value) -> Doc {
    Doc::text_line(format!(
        "{} = const {}",
        namer.name(context, const_val),
        const_val.as_lit_string(context)
    ))
}

fn maybe_constant_to_doc(context: &Context, namer: &mut Namer, maybe_const_val: &Value) -> Doc {
    if maybe_const_val.is_constant(context) {
        constant_to_doc(context, namer, maybe_const_val)
    } else {
        Doc::Empty
    }
}

fn maybe_constant_phi_to_doc(
    context: &Context,
    namer: &mut Namer,
    caller: &Block,
    callee: &Block,
) -> Doc {
    if let ValueContent::Instruction(Instruction::Phi(pairs)) =
        &context.values[callee.get_phi(context).0]
    {
        pairs
            .iter()
            .find(|(block, _)| block == caller)
            .map(|(_, phi_val)| maybe_constant_to_doc(context, namer, phi_val))
            .unwrap_or(Doc::Empty)
    } else {
        unreachable!("Phi must be an instruction.")
    }
}

fn instruction_to_doc<'a>(
    context: &'a Context,
    namer: &mut Namer,
    block: &Block,
    ins_value: &'a Value,
) -> Doc {
    match &context.values[ins_value.0] {
        ValueContent::Instruction(instruction) => match instruction {
            Instruction::AsmBlock(asm, args) => {
                asm_block_to_doc(context, namer, ins_value, asm, args)
            }
            Instruction::Branch(to_block) => {
                maybe_constant_phi_to_doc(context, namer, block, to_block).append(Doc::text_line(
                    format!("br {}", context.blocks[to_block.0].label),
                ))
            }
            Instruction::Call(func, args) => args
                .iter()
                .fold(Doc::Empty, |doc, arg_val| {
                    if arg_val.is_constant(context) {
                        doc.append(constant_to_doc(context, namer, arg_val))
                    } else {
                        doc
                    }
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
                    )),
                )),
            Instruction::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => {
                let true_label = &context.blocks[true_block.0].label;
                let false_label = &context.blocks[false_block.0].label;
                maybe_constant_phi_to_doc(context, namer, block, true_block)
                    .append(maybe_constant_to_doc(context, namer, cond_value))
                    .append(Doc::text_line(format!(
                        "cbr {}, {}, {}",
                        namer.name(context, cond_value),
                        true_label,
                        false_label
                    )))
            }
            Instruction::ExtractElement {
                array,
                ty,
                index_val,
            } => maybe_constant_to_doc(context, namer, index_val).append(Doc::line(Doc::text(
                format!(
                    "{} = extract_element {}, {}, {}",
                    namer.name(context, ins_value),
                    namer.name(context, array),
                    Type::Array(*ty).as_string(context),
                    namer.name(context, index_val),
                ),
            ))),
            Instruction::ExtractValue {
                aggregate,
                ty,
                indices,
            } => Doc::line(
                Doc::text(format!(
                    "{} = extract_value {}, {}, ",
                    namer.name(context, ins_value),
                    namer.name(context, aggregate),
                    Type::Struct(*ty).as_string(context),
                ))
                .append(Doc::list_sep(
                    indices
                        .iter()
                        .map(|idx| Doc::text(format!("{}", idx)))
                        .collect(),
                    Doc::Comma,
                )),
            ),
            Instruction::GetPointer(ptr) => {
                let name = block
                    .get_function(context)
                    .lookup_local_name(context, ptr)
                    .unwrap();
                Doc::text_line(format!(
                    "{} = get_ptr {}",
                    namer.name(context, ins_value),
                    ptr.as_string(context, name)
                ))
            }
            Instruction::InsertElement {
                array,
                ty,
                value,
                index_val,
            } => maybe_constant_to_doc(context, namer, array)
                .append(maybe_constant_to_doc(context, namer, value))
                .append(maybe_constant_to_doc(context, namer, index_val))
                .append(Doc::line(Doc::text(format!(
                    "{} = insert_element {}, {}, {}, {}",
                    namer.name(context, ins_value),
                    namer.name(context, array),
                    Type::Array(*ty).as_string(context),
                    namer.name(context, value),
                    namer.name(context, index_val),
                )))),
            Instruction::InsertValue {
                aggregate,
                ty,
                value,
                indices,
            } => maybe_constant_to_doc(context, namer, aggregate)
                .append(maybe_constant_to_doc(context, namer, value))
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
                            .map(|idx| Doc::text(format!("{}", idx)))
                            .collect(),
                        Doc::Comma,
                    )),
                )),
            Instruction::Load(ptr) => {
                let name = block
                    .get_function(context)
                    .lookup_local_name(context, ptr)
                    .unwrap();
                Doc::text_line(format!(
                    "{} = load {}",
                    namer.name(context, ins_value),
                    ptr.as_string(context, name)
                ))
            }
            Instruction::Phi(pairs) => {
                if pairs.is_empty() {
                    Doc::Empty
                } else {
                    // Name the pairs before we name the PHI instruction itself.
                    let pairs_doc = Doc::in_parens_comma_sep(
                        pairs
                            .iter()
                            .map(|(block, in_value)| {
                                Doc::text(format!(
                                    "{}: {}",
                                    context.blocks[block.0].label,
                                    namer.name(context, in_value)
                                ))
                            })
                            .collect(),
                    );
                    Doc::line(
                        Doc::text(format!("{} = phi", namer.name(context, ins_value)))
                            .append(pairs_doc),
                    )
                }
            }
            Instruction::Ret(v, t) => {
                maybe_constant_to_doc(context, namer, v).append(Doc::text_line(format!(
                    "ret {} {}",
                    t.as_string(context),
                    namer.name(context, v)
                )))
            }
            Instruction::Store { ptr, stored_val } => {
                let name = block
                    .get_function(context)
                    .lookup_local_name(context, ptr)
                    .unwrap();
                maybe_constant_to_doc(context, namer, stored_val).append(Doc::text_line(format!(
                    "store {}, {}",
                    namer.name(context, stored_val),
                    ptr.as_string(context, name),
                )))
            }
        },
        _ => unreachable!("Unexpected non instruction for block contents."),
    }
}

fn asm_block_to_doc(
    context: &Context,
    namer: &mut Namer,
    ins_value: &Value,
    asm: &AsmBlock,
    args: &[AsmArg],
) -> Doc {
    let AsmBlockContent {
        body, return_name, ..
    } = &context.asm_blocks[asm.0];
    args.iter()
        .fold(
            Doc::Empty,
            |doc, AsmArg { initializer, .. }| match initializer {
                Some(init_val) if init_val.is_constant(context) => {
                    doc.append(constant_to_doc(context, namer, init_val))
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
                .append(match return_name {
                    Some(rn) => Doc::text(format!(" -> {} {{", rn)),
                    None => Doc::text(" {"),
                }),
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
                         }| {
                            Doc::line(
                                Doc::text(format!("{:6} ", name.as_str())).append(
                                    Doc::list_sep(
                                        args.iter().map(|arg| Doc::text(arg.as_str())).collect(),
                                        Doc::text(" "),
                                    )
                                    .append(match immediate {
                                        Some(imm_str) => Doc::text(format!(" {}", imm_str)),
                                        None => Doc::Empty,
                                    }),
                                ),
                            )
                        },
                    )
                    .collect(),
            ),
        ))
        .append(Doc::text_line("}"))
}

impl Value {
    fn as_lit_string(&self, context: &Context) -> String {
        if let ValueContent::Constant(c) = &context.values[self.0] {
            c.as_lit_string(context)
        } else {
            unreachable!("Not a literal value.")
        }
    }
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
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<String>>()
                    .concat()
            ),
            ConstantValue::String(s) => format!("{} \"{}\"", self.ty.as_string(context), s),
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

impl Pointer {
    fn as_string(&self, context: &Context, name: &str) -> String {
        let PointerContent { ty, is_mutable, .. } = &context.pointers[self.0];
        let mut_tag = if *is_mutable { "mut " } else { "" };
        format!("{}ptr {} {}", mut_tag, ty.as_string(context), name)
    }

    //fn as_string_no_type(&self, context: &Context, name: &str) -> String {
    //    let PointerContent { is_mutable, .. } = &context.pointers[self.0];
    //    let mut_tag = if *is_mutable { "mut " } else { "" };
    //    format!("{}ptr {}", mut_tag, name)
    //}
}

struct Namer {
    function: Function,
    next_idx: u64,
    names: std::collections::HashMap<Value, String>,
}

impl Namer {
    fn new(function: Function) -> Self {
        Namer {
            function,
            next_idx: 0,
            names: std::collections::HashMap::new(),
        }
    }

    fn name(&mut self, context: &Context, value: &Value) -> String {
        match &context.values[value.0] {
            ValueContent::Argument(_) => self
                .function
                .lookup_arg_name(context, value)
                .cloned()
                .unwrap_or_else(|| self.default_name(value)),
            ValueContent::Constant(_) => self.default_name(value),
            ValueContent::Instruction(_) => self.default_name(value),
        }
    }

    fn default_name(&mut self, value: &Value) -> String {
        self.names.get(value).cloned().unwrap_or_else(|| {
            let new_name = format!("v{}", self.next_idx);
            self.next_idx += 1;
            self.names.insert(*value, new_name.clone());
            new_name
        })
    }
}

/// There will be a much more efficient way to do this, but for now this will do.
fn build_doc(doc: Doc, indent: i64) -> String {
    match doc {
        Doc::Empty => "".into(),
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
