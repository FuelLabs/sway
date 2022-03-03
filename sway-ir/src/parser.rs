//! A parser for the printed IR, useful mostly for testing.

use crate::{context::Context, error::IrError};

// -------------------------------------------------------------------------------------------------
/// Parse a string produced by [`crate::printer::to_string`] into a new [`Context`].
pub fn parse(input: &str) -> Result<Context, IrError> {
    let irmod = ir_builder::parser::ir_descrs(input).map_err(|err| {
        let found = if input.len() - err.location.offset <= 20 {
            &input[err.location.offset..]
        } else {
            &input[err.location.offset..][..20]
        };
        IrError::ParseFailure(err.to_string(), found.into())
    })?;
    ir_builder::build_context(irmod)
}

// -------------------------------------------------------------------------------------------------

mod ir_builder {
    use std::path::PathBuf;

    use sway_types::{ident::Ident, span::Span};

    type MdIdxRef = u64;

    peg::parser! {
        pub(in crate::parser) grammar parser() for str {
            pub(in crate::parser) rule ir_descrs() -> IrAstModule
                = _ s:script() eoi() {
                    s
                }

            rule script() -> IrAstModule
                = "script" _ "{" _ fn_decls:fn_decl()* "}" _ metadata:metadata_decl()* {
                    IrAstModule {
                        kind: crate::module::Kind::Script,
                        fn_decls,
                        metadata
                    }
                }

            rule fn_decl() -> IrAstFnDecl
                = "fn" _ name:id() "(" _ args:(fn_arg() ** comma()) ")" _ "->" _ ret_type:ast_ty() "{" _
                      locals:fn_local()*
                      blocks:block_decl()*
                  "}" _ {
                    IrAstFnDecl {
                        name,
                        args,
                        ret_type,
                        locals,
                        blocks,
                    }
                }

            rule fn_arg() -> (IrAstTy, String, Option<MdIdxRef>)
                = name:id() mdi:metadata_idx()? ":" _ ty:ast_ty() {
                    (ty, name, mdi)
                }

            rule fn_local() -> (IrAstTy, String, bool, Option<IrAstOperation>)
                = "local" _ im:("mut" _)? "ptr" _ ty:ast_ty() name:id() init:fn_local_init()? {
                    (ty, name, im.is_some(), init)
                }

            rule fn_local_init() -> IrAstOperation
                = "=" _ cv:op_const() {
                    cv
                }

            rule block_decl() -> IrAstBlock
                = label:id() ":" _ instructions: instr_decl()+ {
                    IrAstBlock {
                        label,
                        instructions
                    }
                }

            rule instr_decl() -> IrAstInstruction
                = value_name:value_assign()? op:operation() meta_idx:comma_metadata_idx()? {
                    IrAstInstruction {
                        value_name,
                        op,
                        meta_idx,
                    }
                }

            rule value_assign() -> String
                = name:id() "=" _ {
                    name
                }

            rule metadata_idx() -> MdIdxRef
                = "!" i:decimal() {
                    i
                }

            rule comma_metadata_idx() -> MdIdxRef
                = "," _ mdi:metadata_idx() {
                    mdi
                }

            rule operation() -> IrAstOperation
                = op_asm()
                / op_branch()
                / op_call()
                / op_cbr()
                / op_const()
                / op_extract_element()
                / op_extract_value()
                / op_get_ptr()
                / op_insert_element()
                / op_insert_value()
                / op_load()
                / op_nop()
                / op_phi()
                / op_ret()
                / op_store()

            rule op_asm() -> IrAstOperation
                = "asm" _ "(" _ args:(asm_arg() ** comma()) ")" _ ret:asm_ret()? meta_idx:comma_metadata_idx()? "{" _
                    ops:asm_op()*
                "}" _ {
                    IrAstOperation::Asm(args, ret, ops, meta_idx)
                }

            rule op_branch() -> IrAstOperation
                = "br" _ to_block:id() {
                    IrAstOperation::Br(to_block)
                }

            rule op_call() -> IrAstOperation
                = "call" _ callee:id() "(" _ args:(id() ** comma()) ")" _ {
                    IrAstOperation::Call(callee, args)
            }

            rule op_cbr() -> IrAstOperation
                = "cbr" _ cond:id() comma() tblock:id() comma() fblock:id() {
                    IrAstOperation::Cbr(cond, tblock, fblock)
                }

            rule op_const() -> IrAstOperation
                = "const" _ ast_ty() cv:constant() {
                    IrAstOperation::Const(cv)
                }

            rule op_extract_element() -> IrAstOperation
                = "extract_element" _ name:id() comma() ty:ast_ty() comma() idx:id() {
                    IrAstOperation::ExtractElement(name, ty, idx)
                }

            rule op_extract_value() -> IrAstOperation
                = "extract_value" _ name:id() comma() ty:ast_ty() comma() idcs:(decimal() ++ comma()) {
                    IrAstOperation::ExtractValue(name, ty, idcs)
                }

            rule op_get_ptr() -> IrAstOperation
                = "get_ptr" _ ("mut" _)? "ptr" _ ty:ast_ty() name:id() {
                    IrAstOperation::GetPtr(name)
                }

            rule op_insert_element() -> IrAstOperation
                = "insert_element" _ name:id() comma() ty:ast_ty() comma() val:id() comma() idx:id() {
                    IrAstOperation::InsertElement(name, ty, val, idx)
                }

            rule op_insert_value() -> IrAstOperation
                = "insert_value" _ aval:id() comma() ty:ast_ty() comma() ival:id() comma() idcs:(decimal() ++ comma()) {
                    IrAstOperation::InsertValue(aval, ty, ival, idcs)
                }

            rule op_load() -> IrAstOperation
                = "load" _ ("mut" _)? "ptr" _ ast_ty() src:id() {
                    IrAstOperation::Load(src)
                }

            rule op_nop() -> IrAstOperation
                = "nop" _ {
                    IrAstOperation::Nop
                }

            rule op_phi() -> IrAstOperation
                = "phi" _ "(" _ pairs:((bl:id() ":" _ vn:id() { (bl, vn) }) ** comma()) ")" _ {
                    IrAstOperation::Phi(pairs)
                }

            rule op_ret() -> IrAstOperation
                = "ret" _ ty:ast_ty() vn:id() {
                    IrAstOperation::Ret(ty, vn)
                }

            rule op_store() -> IrAstOperation
                = "store" _ dst:id() comma() ("mut" _)? "ptr" _ ast_ty() vn:id() {
                    IrAstOperation::Store(dst, vn)
                }

            rule asm_arg() -> (Ident, Option<IrAstAsmArgInit>)
                = name:id_id() init:asm_arg_init()? {
                    (name, init)
            }

            rule asm_arg_init() -> IrAstAsmArgInit
                = ":" _ imm:constant() {
                    IrAstAsmArgInit::Imm(imm)
                }
                / ":" _ var:id() {
                    IrAstAsmArgInit::Var(var)
                }

            rule asm_ret() -> Ident
                = "->" _ ret:id_id() {
                    ret
                }

            rule asm_op() -> IrAstAsmOp
                = name:id_id() args:asm_op_arg()* imm:asm_op_arg_imm()? meta_idx:comma_metadata_idx()? {
                    IrAstAsmOp {
                        name,
                        args,
                        imm,
                        meta_idx
                    }
                }

            rule asm_op_arg() -> Ident
                = !asm_op_arg_imm() arg:id_id() {
                    arg
                }

            rule asm_op_arg_imm() -> Ident
                = imm:$("i" d:decimal()) {
                    Ident::new(Span {
                        span: pest::Span::new(imm.into(), 0, imm.len()).unwrap(),
                        path: None,
                    })
                }

            rule constant() -> IrAstConst
                = value:constant_value() meta_idx:metadata_idx()? {
                    IrAstConst {
                        value,
                        meta_idx
                    }
                }

            rule constant_value() -> IrAstConstValue
                = "()" _ { IrAstConstValue::Unit }
                / "true" _ { IrAstConstValue::Bool(true) }
                / "false" _ { IrAstConstValue::Bool(false) }
                / "0x" s:$(['0'..='9' | 'a'..='f' | 'A'..='F']*<64>) _ {
                    let mut bytes: [u8; 32] = [0; 32];
                    let mut cur_byte: u8 = 0;
                    for (idx, ch) in s.chars().enumerate() {
                        cur_byte = (cur_byte << 4) | ch.to_digit(16).unwrap() as u8;
                        if idx % 2 == 1 {
                            bytes[idx / 2] = cur_byte;
                            cur_byte = 0;
                        }
                    }
                    IrAstConstValue::B256(bytes)
                }
                / n:decimal() { IrAstConstValue::Number(n) }
                / string_const()
                / array_const()
                / struct_const()

            rule string_const() -> IrAstConstValue
                = ['"'] chs:$(str_char()*) ['"'] _ {
                    IrAstConstValue::String(chs.to_owned())
                }

            rule str_char()
                = [^ '"' | '\\'] / ['\\'] ['\\' | 't' | 'n' | 'r']

            rule array_const() -> IrAstConstValue
                = "[" _ els:(field_or_element_const() ++ comma()) "]" _ {
                    let el_ty = els[0].0.clone();
                    let els = els.into_iter().map(|(_, cv)| cv).collect::<Vec<_>>();
                    IrAstConstValue::Array(el_ty, els)
                }

            rule struct_const() -> IrAstConstValue
                = "{" _ flds:(field_or_element_const() ++ comma()) "}" _ {
                    IrAstConstValue::Struct(flds)
                }

            rule field_or_element_const() -> (IrAstTy, IrAstConst)
                = ty:ast_ty() cv:constant() {
                    (ty, cv)
                }
                / ty:ast_ty() "undef" _ {
                    (ty.clone(), IrAstConst { value: IrAstConstValue::Undef(ty), meta_idx: None })
                }

            rule ast_ty() -> IrAstTy
                = ("unit" / "()") _ { IrAstTy::Unit }
                / "bool" _ { IrAstTy::Bool }
                / "u64" _ { IrAstTy::U64 }
                / "b256" _ { IrAstTy::B256 }
                / "string" _ "<" _ sz:decimal() ">" _ { IrAstTy::String(sz) }
                / array_ty()
                / enum_ty()
                / struct_ty()

            rule array_ty() -> IrAstTy
                = "[" _ ty:ast_ty() ";" _ c:decimal() "]" _ {
                    IrAstTy::Array(Box::new(ty), c)
                }

            rule enum_ty() -> IrAstTy
                = "{" _ tys:(ast_ty() ++ ("|" _)) "}" _ {
                    IrAstTy::Union(tys)
                }

            rule struct_ty() -> IrAstTy
                = "{" _ tys:(ast_ty() ++ comma()) "}" _ {
                    IrAstTy::Struct(tys)
                }

            rule id() -> String
                = !ast_ty() id:$(id_char0() id_char()*) _ {
                    id.to_owned()
                }

            rule id_id() -> Ident
                = !ast_ty() id:$(id_char0() id_char()*) _ {
                    Ident::new(Span {
                        span: pest::Span::new(id.into(), 0, id.len()).unwrap(),
                        path: None,
                    })
                }

            rule metadata_decl() -> (MdIdxRef, IrMetadatum)
                = "!" idx:decimal() "=" _ item:metadata_item() {
                    (idx, item)
                }

            rule metadata_item() -> IrMetadatum
                = "filepath" _ ['"'] path:$(([^ '"' | '\\'] / ['\\'] ['\\' | '"' ])+) ['"'] _ {
                    IrMetadatum::FilePath(PathBuf::from(path))
                }
                / "span" _ "!" li:decimal() s:decimal() e:decimal() {
                    IrMetadatum::Span { loc_idx: li, start: s as usize, end: e as usize }
                }

            rule id_char0()
                = quiet!{ ['A'..='Z' | 'a'..='z' | '_'] }

            rule id_char()
                = quiet!{ id_char0() / ['0'..='9'] }

            rule decimal() -> u64
                = ds:$("0" / ['1'..='9'] ['0'..='9']*) _ {
                    ds.parse::<u64>().unwrap()
                }

            rule comma()
                = quiet!{ "," _ }

            rule _()
                = quiet!{ (ws() / comment())* }

            rule ws()
                = [' ' | '\t']
                / nl()

            rule nl()
                = ['\n' | '\r']

            rule comment()
                = "//" (!nl() [_])* nl()

            rule eoi()
                = ![_] / expected!("end of input")
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    use crate::{
        asm::{AsmArg, AsmInstruction},
        block::Block,
        constant::Constant,
        context::Context,
        error::IrError,
        function::Function,
        instruction::Instruction,
        irtype::{Aggregate, Type},
        metadata::{MetadataIndex, Metadatum},
        module::{Kind, Module},
        pointer::Pointer,
        value::Value,
    };

    #[derive(Debug)]
    pub(super) struct IrAstModule {
        kind: Kind,
        fn_decls: Vec<IrAstFnDecl>,
        metadata: Vec<(MdIdxRef, IrMetadatum)>,
    }

    #[derive(Debug)]
    struct IrAstFnDecl {
        name: String,
        args: Vec<(IrAstTy, String, Option<MdIdxRef>)>,
        ret_type: IrAstTy,
        locals: Vec<(IrAstTy, String, bool, Option<IrAstOperation>)>,
        blocks: Vec<IrAstBlock>,
    }

    #[derive(Debug)]
    struct IrAstBlock {
        label: String,
        instructions: Vec<IrAstInstruction>,
    }

    #[derive(Debug)]
    struct IrAstInstruction {
        value_name: Option<String>,
        op: IrAstOperation,
        meta_idx: Option<MdIdxRef>,
    }

    #[derive(Debug)]
    enum IrAstOperation {
        Asm(
            Vec<(Ident, Option<IrAstAsmArgInit>)>,
            Option<Ident>,
            Vec<IrAstAsmOp>,
            Option<MdIdxRef>,
        ),
        Br(String),
        Call(String, Vec<String>),
        Cbr(String, String, String),
        Const(IrAstConst),
        ExtractElement(String, IrAstTy, String),
        ExtractValue(String, IrAstTy, Vec<u64>),
        GetPtr(String),
        InsertElement(String, IrAstTy, String, String),
        InsertValue(String, IrAstTy, String, Vec<u64>),
        Load(String),
        Nop,
        Phi(Vec<(String, String)>),
        Ret(IrAstTy, String),
        Store(String, String),
    }

    #[derive(Debug)]
    struct IrAstConst {
        value: IrAstConstValue,
        meta_idx: Option<MdIdxRef>,
    }

    #[derive(Debug)]
    enum IrAstConstValue {
        Undef(IrAstTy),
        Unit,
        Bool(bool),
        B256([u8; 32]),
        Number(u64),
        String(String),
        Array(IrAstTy, Vec<IrAstConst>),
        Struct(Vec<(IrAstTy, IrAstConst)>),
    }

    #[derive(Debug)]
    enum IrAstAsmArgInit {
        Var(String),
        Imm(IrAstConst),
    }

    #[derive(Debug)]
    struct IrAstAsmOp {
        name: Ident,
        args: Vec<Ident>,
        imm: Option<Ident>,
        meta_idx: Option<MdIdxRef>,
    }

    impl IrAstConstValue {
        fn as_constant(&self, context: &mut Context) -> Constant {
            match self {
                IrAstConstValue::Undef(ty) => {
                    let ty = ty.to_ir_type(context);
                    Constant::new_undef(context, ty)
                }
                IrAstConstValue::Unit => Constant::new_unit(),
                IrAstConstValue::Bool(b) => Constant::new_bool(*b),
                IrAstConstValue::B256(bs) => Constant::new_b256(*bs),
                IrAstConstValue::Number(n) => Constant::new_uint(64, *n),
                IrAstConstValue::String(s) => Constant::new_string(s.clone()),
                IrAstConstValue::Array(el_ty, els) => {
                    let els: Vec<_> = els.iter().map(|cv| cv.value.as_constant(context)).collect();
                    let el_ty = el_ty.to_ir_type(context);
                    let array = Aggregate::new_array(context, el_ty, els.len() as u64);
                    Constant::new_array(&array, els)
                }
                IrAstConstValue::Struct(flds) => {
                    // To Make a Constant I need to create an aggregate, which requires a context.
                    let (types, fields): (Vec<_>, Vec<_>) = flds
                        .iter()
                        .map(|(ty, cv)| (ty.to_ir_type(context), cv.value.as_constant(context)))
                        .unzip();
                    let aggregate = Aggregate::new_struct(context, types);
                    Constant::new_struct(&aggregate, fields)
                }
            }
        }

        fn as_value(&self, context: &mut Context, span_md_idx: Option<MetadataIndex>) -> Value {
            match self {
                IrAstConstValue::Undef(_) => unreachable!("Can't convert 'undef' to a value."),
                IrAstConstValue::Unit => Constant::get_unit(context, span_md_idx),
                IrAstConstValue::Bool(b) => Constant::get_bool(context, *b, span_md_idx),
                IrAstConstValue::B256(bs) => Constant::get_b256(context, *bs, span_md_idx),
                IrAstConstValue::Number(n) => Constant::get_uint(context, 64, *n, span_md_idx),
                IrAstConstValue::String(s) => Constant::get_string(context, s.clone(), span_md_idx),
                IrAstConstValue::Array(..) => {
                    let array_const = self.as_constant(context);
                    Constant::get_array(context, array_const, span_md_idx)
                }
                IrAstConstValue::Struct(_) => {
                    let struct_const = self.as_constant(context);
                    Constant::get_struct(context, struct_const, span_md_idx)
                }
            }
        }
    }

    #[derive(Clone, Debug)]
    enum IrAstTy {
        Unit,
        Bool,
        U64,
        B256,
        String(u64),
        Array(Box<IrAstTy>, u64),
        Union(Vec<IrAstTy>),
        Struct(Vec<IrAstTy>),
    }

    impl IrAstTy {
        fn to_ir_type(&self, context: &mut Context) -> Type {
            match self {
                IrAstTy::Unit => Type::Unit,
                IrAstTy::Bool => Type::Bool,
                IrAstTy::U64 => Type::Uint(64),
                IrAstTy::B256 => Type::B256,
                IrAstTy::String(n) => Type::String(*n),
                IrAstTy::Array(..) => Type::Array(self.to_ir_aggregate_type(context)),
                IrAstTy::Union(_) => Type::Union(self.to_ir_aggregate_type(context)),
                IrAstTy::Struct(_) => Type::Struct(self.to_ir_aggregate_type(context)),
            }
        }

        fn to_ir_aggregate_type(&self, context: &mut Context) -> Aggregate {
            match self {
                IrAstTy::Array(el_ty, count) => {
                    let el_ty = el_ty.to_ir_type(context);
                    Aggregate::new_array(context, el_ty, *count)
                }
                IrAstTy::Struct(tys) | IrAstTy::Union(tys) => {
                    let tys = tys.iter().map(|ty| ty.to_ir_type(context)).collect();
                    Aggregate::new_struct(context, tys)
                }
                _otherwise => {
                    unreachable!("Converting non aggregate IR AST type to IR aggregate type.")
                }
            }
        }
    }

    #[derive(Debug)]
    enum IrMetadatum {
        FilePath(PathBuf),
        Span {
            loc_idx: MdIdxRef,
            start: usize,
            end: usize,
        },
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    use std::{collections::HashMap, iter::FromIterator, sync::Arc};

    pub(super) fn build_context(ir_ast_mod: IrAstModule) -> Result<Context, IrError> {
        let mut ctx = Context::default();
        let module = Module::new(&mut ctx, ir_ast_mod.kind);
        let md_map = build_metadata_map(&mut ctx, &ir_ast_mod.metadata);
        let mut unresolved_calls = Vec::new();
        for fn_decl in ir_ast_mod.fn_decls {
            build_add_fn_decl(&mut ctx, module, fn_decl, &md_map, &mut unresolved_calls)?;
        }
        resolve_calls(&mut ctx, unresolved_calls)?;
        Ok(ctx)
    }

    #[allow(clippy::type_complexity)]
    fn build_add_fn_decl(
        context: &mut Context,
        module: Module,
        fn_decl: IrAstFnDecl,
        md_map: &HashMap<MdIdxRef, MetadataIndex>,
        unresolved_calls: &mut Vec<(Block, Value, String, Vec<Value>, Option<MetadataIndex>)>,
    ) -> Result<(), IrError> {
        let args: Vec<(String, Type, Option<MetadataIndex>)> = fn_decl
            .args
            .iter()
            .map(|(ty, name, md_idx)| {
                (
                    name.into(),
                    ty.to_ir_type(context),
                    md_idx.map(|mdi| md_map.get(&mdi).copied().unwrap()),
                )
            })
            .collect();
        let ret_type = fn_decl.ret_type.to_ir_type(context);
        let func = Function::new(
            context,
            module,
            fn_decl.name,
            args.clone(),
            ret_type,
            None,
            false,
        );

        // Gather all the (new) arg values by name into a map.
        let mut arg_map: HashMap<String, Value> =
            HashMap::from_iter(args.into_iter().map(|(name, _, _)| {
                let arg_val = func.get_arg(context, &name).unwrap();
                (name, arg_val)
            }));
        let mut ptr_map = HashMap::<String, Pointer>::new();
        for (ty, name, is_mutable, initializer) in fn_decl.locals {
            let initializer = initializer.map(|const_init| {
                if let IrAstOperation::Const(val) = const_init {
                    val.value.as_constant(context)
                } else {
                    unreachable!("BUG! Initializer must be a const value.");
                }
            });
            let ty = ty.to_ir_type(context);
            ptr_map.insert(
                name.clone(),
                func.new_local_ptr(context, name, ty, is_mutable, initializer)?,
            );
        }

        // The entry block is already created, we don't want to recrate it.
        let named_blocks = HashMap::from_iter(fn_decl.blocks.iter().map(|block| {
            (
                block.label.clone(),
                if block.label == "entry" {
                    func.get_entry_block(context)
                } else {
                    func.create_block(context, Some(block.label.clone()))
                },
            )
        }));

        for block in fn_decl.blocks {
            build_add_block_instructions(
                context,
                block,
                &named_blocks,
                &ptr_map,
                &mut arg_map,
                md_map,
                unresolved_calls,
            );
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn build_add_block_instructions(
        context: &mut Context,
        ir_block: IrAstBlock,
        named_blocks: &HashMap<String, Block>,
        ptr_map: &HashMap<String, Pointer>,
        val_map: &mut HashMap<String, Value>,
        md_map: &HashMap<MdIdxRef, MetadataIndex>,
        unresolved_calls: &mut Vec<(Block, Value, String, Vec<Value>, Option<MetadataIndex>)>,
    ) {
        let block = named_blocks.get(&ir_block.label).unwrap();
        for ins in ir_block.instructions {
            let opt_ins_md_idx = ins.meta_idx.map(|mdi| md_map.get(&mdi).unwrap()).copied();
            let ins_val = match ins.op {
                IrAstOperation::Asm(args, return_name, ops, meta_idx) => {
                    let args = args
                        .into_iter()
                        .map(|(name, opt_init)| AsmArg {
                            name,
                            initializer: opt_init.map(|init| match init {
                                IrAstAsmArgInit::Var(var) => val_map.get(&var).cloned().unwrap(),
                                IrAstAsmArgInit::Imm(cv) => cv.value.as_value(
                                    context,
                                    md_map.get(cv.meta_idx.as_ref().unwrap()).copied(),
                                ),
                            }),
                        })
                        .collect();
                    let body = ops
                        .into_iter()
                        .map(
                            |IrAstAsmOp {
                                 name,
                                 args,
                                 imm,
                                 meta_idx,
                             }| AsmInstruction {
                                name,
                                args,
                                immediate: imm,
                                span_md_idx: meta_idx
                                    .as_ref()
                                    .and_then(|meta_idx| md_map.get(meta_idx).copied()),
                            },
                        )
                        .collect();
                    let md_idx = meta_idx.map(|mdi| md_map.get(&mdi).unwrap()).copied();
                    block
                        .ins(context)
                        .asm_block(args, body, return_name, md_idx)
                }
                IrAstOperation::Br(to_block_name) => {
                    let to_block = named_blocks.get(&to_block_name).unwrap();
                    block.ins(context).branch(*to_block, None, opt_ins_md_idx)
                }
                IrAstOperation::Call(callee, args) => {
                    // We can't resolve calls to other functions until we've done a first pass and
                    // created them first.  So we can insert a NOP here, save the call params and
                    // replace it with a CALL in a second pass.
                    let nop = block.ins(context).nop();
                    unresolved_calls.push((
                        *block,
                        nop,
                        callee,
                        args.iter()
                            .map(|arg_name| val_map.get(arg_name).unwrap())
                            .cloned()
                            .collect::<Vec<Value>>(),
                        opt_ins_md_idx,
                    ));
                    nop
                }
                IrAstOperation::Cbr(cond_val_name, true_block_name, false_block_name) => {
                    block.ins(context).conditional_branch(
                        *val_map.get(&cond_val_name).unwrap(),
                        *named_blocks.get(&true_block_name).unwrap(),
                        *named_blocks.get(&false_block_name).unwrap(),
                        None,
                        opt_ins_md_idx,
                    )
                }
                IrAstOperation::Const(val) => val.value.as_value(context, opt_ins_md_idx),
                IrAstOperation::ExtractElement(aval, ty, idx) => {
                    let ir_ty = ty.to_ir_aggregate_type(context);
                    block.ins(context).extract_element(
                        *val_map.get(&aval).unwrap(),
                        ir_ty,
                        *val_map.get(&idx).unwrap(),
                        opt_ins_md_idx,
                    )
                }
                IrAstOperation::ExtractValue(val, ty, idcs) => {
                    let ir_ty = ty.to_ir_aggregate_type(context);
                    block.ins(context).extract_value(
                        *val_map.get(&val).unwrap(),
                        ir_ty,
                        idcs,
                        opt_ins_md_idx,
                    )
                }
                IrAstOperation::GetPtr(src_name) => block
                    .ins(context)
                    .get_ptr(*ptr_map.get(&src_name).unwrap(), opt_ins_md_idx),
                IrAstOperation::InsertElement(aval, ty, val, idx) => {
                    let ir_ty = ty.to_ir_aggregate_type(context);
                    block.ins(context).insert_element(
                        *val_map.get(&aval).unwrap(),
                        ir_ty,
                        *val_map.get(&val).unwrap(),
                        *val_map.get(&idx).unwrap(),
                        opt_ins_md_idx,
                    )
                }
                IrAstOperation::InsertValue(aval, ty, ival, idcs) => {
                    let ir_ty = ty.to_ir_aggregate_type(context);
                    block.ins(context).insert_value(
                        *val_map.get(&aval).unwrap(),
                        ir_ty,
                        *val_map.get(&ival).unwrap(),
                        idcs,
                        opt_ins_md_idx,
                    )
                }
                IrAstOperation::Load(src_name) => block
                    .ins(context)
                    .load(*ptr_map.get(&src_name).unwrap(), opt_ins_md_idx),
                IrAstOperation::Nop => block.ins(context).nop(),
                IrAstOperation::Phi(pairs) => {
                    for (block_name, val_name) in pairs {
                        block.add_phi(
                            context,
                            *named_blocks.get(&block_name).unwrap(),
                            *val_map.get(&val_name).unwrap(),
                        );
                    }
                    block.get_phi(context)
                }
                IrAstOperation::Ret(ty, ret_val_name) => {
                    let ty = ty.to_ir_type(context);
                    block
                        .ins(context)
                        .ret(*val_map.get(&ret_val_name).unwrap(), ty, opt_ins_md_idx)
                }
                IrAstOperation::Store(stored_val_name, ptr_name) => block.ins(context).store(
                    *ptr_map.get(&ptr_name).unwrap(),
                    *val_map.get(&stored_val_name).unwrap(),
                    opt_ins_md_idx,
                ),
            };
            ins.value_name.map(|vn| val_map.insert(vn, ins_val));
        }
    }

    fn build_metadata_map(
        context: &mut Context,
        ir_metadata: &[(MdIdxRef, IrMetadatum)],
    ) -> HashMap<MdIdxRef, MetadataIndex> {
        let mut md_map = ir_metadata
            .iter()
            .filter_map(|(idx_ref, md)| match md {
                IrMetadatum::FilePath(path) => Some((idx_ref, path)),
                _otherwise => None,
            })
            .fold(HashMap::new(), |mut md_map, (idx_ref, path)| {
                let path_content = Arc::from(std::fs::read_to_string(path).unwrap().as_str());
                let md_idx = context.metadata.insert(Metadatum::FileLocation(
                    Arc::new(path.clone()),
                    path_content,
                ));
                md_map.insert(*idx_ref, MetadataIndex(md_idx));
                md_map
            });

        for (idx_ref, md) in ir_metadata {
            if let IrMetadatum::Span {
                loc_idx,
                start,
                end,
            } = md
            {
                let span_idx = context.metadata.insert(Metadatum::Span {
                    loc_idx: md_map.get(loc_idx).copied().unwrap(),
                    start: *start,
                    end: *end,
                });
                md_map.insert(*idx_ref, MetadataIndex(span_idx));
            }
        }
        md_map
    }

    #[allow(clippy::type_complexity)]
    fn resolve_calls(
        context: &mut Context,
        unresolved_calls: Vec<(Block, Value, String, Vec<Value>, Option<MetadataIndex>)>,
    ) -> Result<(), IrError> {
        // All of the call instructions are currently NOPs which need to be replaced with actual
        // calls.  We couldn't do it above until we'd gone and created all the functions first.
        //
        // Now we can loop and find the callee function for each call and replace the NOPs.
        for (block, nop, callee, args, opt_ins_md_idx) in unresolved_calls {
            let function = context
                .functions
                .iter()
                .find_map(|(idx, content)| {
                    if content.name == callee {
                        Some(Function(idx))
                    } else {
                        None
                    }
                })
                .unwrap();
            let call_val =
                Value::new_instruction(context, Instruction::Call(function, args), opt_ins_md_idx);
            block.replace_instruction(context, nop, call_val)?;
        }
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------
