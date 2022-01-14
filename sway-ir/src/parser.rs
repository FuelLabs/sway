//! A parser for the printed IR, useful mostly for testing.

use crate::context::Context;

// -------------------------------------------------------------------------------------------------
/// Parse a string produced by [`crate::printer::to_string`] into a new [`Context`].
pub fn parse(input: &str) -> Result<Context, String> {
    let irmod = ir_builder::parser::ir_descrs(input).map_err(|err| {
        let found = if input.len() - err.location.offset <= 20 {
            &input[err.location.offset..]
        } else {
            &input[err.location.offset..][..20]
        };
        format!("parse failed: {}, found: {}", err, found)
    })?;
    ir_builder::build_context(irmod)
}

// -------------------------------------------------------------------------------------------------

mod ir_builder {
    use sway_types::{ident::Ident, span::Span};

    peg::parser! {
        pub(in crate::parser) grammar parser() for str {
            pub(in crate::parser) rule ir_descrs() -> IrAstModule
                = _ s:script() eoi() {
                    s
                }

            rule script() -> IrAstModule
                = "script" _ name:id() "{" _ fn_decls:fn_decl()* "}" _ {
                    IrAstModule {
                        name,
                        kind: crate::module::Kind::Script,
                        fn_decls
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

            rule fn_arg() -> (IrAstTy, String)
                = name:id() ":" _ ty:ast_ty() {
                    (ty, name)
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
                = value_name:value_assign()? op:operation() {
                    IrAstInstruction {
                        value_name,
                        op,
                    }
                }

            rule value_assign() -> String
                = name:id() "=" _ {
                    name
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
                / op_phi()
                / op_ret()
                / op_store()

            rule op_asm() -> IrAstOperation
                = "asm" _ "(" _ args:(asm_arg() ** comma()) ")" _ ret:asm_ret()? "{" _
                    ops:asm_op()*
                "}" _ {
                    IrAstOperation::Asm(args, ret, ops)
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
                = name:id_id() args:asm_op_arg()* imm:asm_op_arg_imm()? {
                    IrAstAsmOp {
                        name,
                        args,
                        imm
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

            rule constant() -> IrAstConstValue
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
                    let els = els.into_iter().map(|(_, cv)| cv).collect();
                    IrAstConstValue::Array(el_ty, els)
                }

            rule struct_const() -> IrAstConstValue
                = "{" _ flds:(field_or_element_const() ++ comma()) "}" _ {
                    IrAstConstValue::Struct(flds)
                }

            rule field_or_element_const() -> (IrAstTy, IrAstConstValue)
                = ty:ast_ty() cv:constant() {
                    (ty, cv)
                }
                / ty:ast_ty() "undef" _ {
                    (ty.clone(), IrAstConstValue::Undef(ty))
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
        function::Function,
        irtype::{Aggregate, Type},
        module::{Kind, Module},
        pointer::Pointer,
        value::Value,
    };

    #[derive(Debug)]
    pub(super) struct IrAstModule {
        name: String,
        kind: Kind,
        fn_decls: Vec<IrAstFnDecl>,
    }

    #[derive(Debug)]
    struct IrAstFnDecl {
        name: String,
        args: Vec<(IrAstTy, String)>,
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
    }

    #[derive(Debug)]
    enum IrAstOperation {
        Asm(
            Vec<(Ident, Option<IrAstAsmArgInit>)>,
            Option<Ident>,
            Vec<IrAstAsmOp>,
        ),
        Br(String),
        Call(String, Vec<String>),
        Cbr(String, String, String),
        Const(IrAstConstValue),
        ExtractElement(String, IrAstTy, String),
        ExtractValue(String, IrAstTy, Vec<u64>),
        GetPtr(String),
        InsertElement(String, IrAstTy, String, String),
        InsertValue(String, IrAstTy, String, Vec<u64>),
        Load(String),
        Phi(Vec<(String, String)>),
        Ret(IrAstTy, String),
        Store(String, String),
    }

    #[derive(Debug)]
    enum IrAstConstValue {
        Undef(IrAstTy),
        Unit,
        Bool(bool),
        B256([u8; 32]),
        Number(u64),
        String(String),
        Array(IrAstTy, Vec<IrAstConstValue>),
        Struct(Vec<(IrAstTy, IrAstConstValue)>),
    }

    #[derive(Debug)]
    enum IrAstAsmArgInit {
        Var(String),
        Imm(IrAstConstValue),
    }

    #[derive(Debug)]
    struct IrAstAsmOp {
        name: Ident,
        args: Vec<Ident>,
        imm: Option<Ident>,
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
                    let els: Vec<_> = els.iter().map(|cv| cv.as_constant(context)).collect();
                    let el_ty = el_ty.to_ir_type(context);
                    let array = Aggregate::new_array(context, el_ty, els.len() as u64);
                    Constant::new_array(&array, els)
                }
                IrAstConstValue::Struct(flds) => {
                    // To Make a Constant I need to create an aggregate, which requires a context.
                    let (types, fields): (Vec<_>, Vec<_>) = flds
                        .iter()
                        .map(|(ty, cv)| (ty.to_ir_type(context), cv.as_constant(context)))
                        .unzip();
                    let aggregate = Aggregate::new_struct(context, None, types);
                    Constant::new_struct(&aggregate, fields)
                }
            }
        }

        fn as_value(&self, context: &mut Context) -> Value {
            match self {
                IrAstConstValue::Undef(_) => unreachable!("Can't convert 'undef' to a value."),
                IrAstConstValue::Unit => Constant::get_unit(context),
                IrAstConstValue::Bool(b) => Constant::get_bool(context, *b),
                IrAstConstValue::B256(bs) => Constant::get_b256(context, *bs),
                IrAstConstValue::Number(n) => Constant::get_uint(context, 64, *n),
                IrAstConstValue::String(s) => Constant::get_string(context, s.clone()),
                IrAstConstValue::Array(..) => {
                    let array_const = self.as_constant(context);
                    Constant::get_array(context, array_const)
                }
                IrAstConstValue::Struct(_) => {
                    let struct_const = self.as_constant(context);
                    Constant::get_struct(context, struct_const)
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
                    Aggregate::new_struct(context, None, tys)
                }
                _otherwise => {
                    unreachable!("Converting non aggregate IR AST type to IR aggregate type.")
                }
            }
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    use std::collections::HashMap;
    use std::iter::FromIterator;

    pub(super) fn build_context(ir_ast_mod: IrAstModule) -> Result<Context, String> {
        let mut ctx = Context::default();
        let module = Module::new(&mut ctx, ir_ast_mod.kind, &ir_ast_mod.name);
        for fn_decl in ir_ast_mod.fn_decls {
            build_add_fn_decl(&mut ctx, module, fn_decl)?;
        }
        Ok(ctx)
    }

    fn build_add_fn_decl(
        context: &mut Context,
        module: Module,
        fn_decl: IrAstFnDecl,
    ) -> Result<(), String> {
        let args: Vec<(String, Type)> = fn_decl
            .args
            .iter()
            .map(|(ty, name)| (name.into(), ty.to_ir_type(context)))
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
            HashMap::from_iter(args.into_iter().map(|(name, _)| {
                let arg_val = func.get_arg(context, &name).unwrap();
                (name, arg_val)
            }));
        let mut ptr_map = HashMap::<String, Pointer>::new();
        for (ty, name, is_mutable, initializer) in fn_decl.locals {
            let initializer = initializer.map(|const_init| {
                if let IrAstOperation::Const(val) = const_init {
                    val.as_constant(context)
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
            build_add_block_instructions(context, block, &named_blocks, &ptr_map, &mut arg_map);
        }
        Ok(())
    }

    fn build_add_block_instructions(
        context: &mut Context,
        ir_block: IrAstBlock,
        named_blocks: &HashMap<String, Block>,
        ptr_map: &HashMap<String, Pointer>,
        val_map: &mut HashMap<String, Value>,
    ) {
        let block = named_blocks.get(&ir_block.label).unwrap();
        for ins in ir_block.instructions {
            let ins_val = match ins.op {
                IrAstOperation::Asm(args, return_name, ops) => {
                    let args = args
                        .into_iter()
                        .map(|(name, opt_init)| AsmArg {
                            name,
                            initializer: opt_init.map(|init| match init {
                                IrAstAsmArgInit::Var(var) => val_map.get(&var).cloned().unwrap(),
                                IrAstAsmArgInit::Imm(cv) => cv.as_value(context),
                            }),
                        })
                        .collect();
                    let body = ops
                        .into_iter()
                        .map(|IrAstAsmOp { name, args, imm }| AsmInstruction {
                            name,
                            args,
                            immediate: imm, //: Option<String>,
                        })
                        .collect();
                    block.ins(context).asm_block(args, body, return_name)
                }
                IrAstOperation::Br(to_block_name) => {
                    let to_block = named_blocks.get(&to_block_name).unwrap();
                    block.ins(context).branch(*to_block, None)
                }
                IrAstOperation::Call(callee, args) => {
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
                    block.ins(context).call(
                        function,
                        &args
                            .iter()
                            .map(|arg_name| val_map.get(arg_name).unwrap())
                            .cloned()
                            .collect::<Vec<Value>>(),
                    )
                }
                IrAstOperation::Cbr(cond_val_name, true_block_name, false_block_name) => {
                    block.ins(context).conditional_branch(
                        *val_map.get(&cond_val_name).unwrap(),
                        *named_blocks.get(&true_block_name).unwrap(),
                        *named_blocks.get(&false_block_name).unwrap(),
                        None,
                    )
                }
                IrAstOperation::Const(val) => val.as_value(context),
                IrAstOperation::ExtractElement(aval, ty, idx) => {
                    let ir_ty = ty.to_ir_aggregate_type(context);
                    block.ins(context).extract_element(
                        *val_map.get(&aval).unwrap(),
                        ir_ty,
                        *val_map.get(&idx).unwrap(),
                    )
                }
                IrAstOperation::ExtractValue(val, ty, idcs) => {
                    let ir_ty = ty.to_ir_aggregate_type(context);
                    block
                        .ins(context)
                        .extract_value(*val_map.get(&val).unwrap(), ir_ty, idcs)
                }
                IrAstOperation::GetPtr(src_name) => {
                    block.ins(context).get_ptr(*ptr_map.get(&src_name).unwrap())
                }
                IrAstOperation::InsertElement(aval, ty, val, idx) => {
                    let ir_ty = ty.to_ir_aggregate_type(context);
                    block.ins(context).insert_element(
                        *val_map.get(&aval).unwrap(),
                        ir_ty,
                        *val_map.get(&val).unwrap(),
                        *val_map.get(&idx).unwrap(),
                    )
                }
                IrAstOperation::InsertValue(aval, ty, ival, idcs) => {
                    let ir_ty = ty.to_ir_aggregate_type(context);
                    block.ins(context).insert_value(
                        *val_map.get(&aval).unwrap(),
                        ir_ty,
                        *val_map.get(&ival).unwrap(),
                        idcs,
                    )
                }
                IrAstOperation::Load(src_name) => {
                    block.ins(context).load(*ptr_map.get(&src_name).unwrap())
                }
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
                        .ret(*val_map.get(&ret_val_name).unwrap(), ty)
                }
                IrAstOperation::Store(stored_val_name, ptr_name) => block.ins(context).store(
                    *ptr_map.get(&ptr_name).unwrap(),
                    *val_map.get(&stored_val_name).unwrap(),
                ),
            };
            ins.value_name.map(|vn| val_map.insert(vn, ins_val));
        }
    }
}

// -------------------------------------------------------------------------------------------------
