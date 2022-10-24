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
    ir_builder::build_context(irmod)?.verify()
}

// -------------------------------------------------------------------------------------------------

mod ir_builder {
    use sway_types::{ident::Ident, span::Span};

    type MdIdxRef = u64;

    peg::parser! {
        pub(in crate::parser) grammar parser() for str {
            pub(in crate::parser) rule ir_descrs() -> IrAstModule
                = _ s:script() eoi() {
                    s
                }
                / _ c:contract() eoi() {
                    c
                }

            rule script() -> IrAstModule
                = "script" _ "{" _ fn_decls:fn_decl()* "}" _ metadata:metadata_decls() {
                    IrAstModule {
                        kind: crate::module::Kind::Script,
                        fn_decls,
                        metadata
                    }
                }

            rule contract() -> IrAstModule
                = "contract" _ "{" _ fn_decls:fn_decl()* "}" _ metadata:metadata_decls() {
                    IrAstModule {
                        kind: crate::module::Kind::Contract,
                        fn_decls,
                        metadata
                    }
                }

            rule fn_decl() -> IrAstFnDecl
                = "pub fn" _ name:id() _ selector:selector_id()? _ "(" _
                        args:(block_arg() ** comma()) ")" _ "->" _ ret_type:ast_ty()
                            metadata:comma_metadata_idx()? "{" _
                        locals:fn_local()*
                        blocks:block_decl()*
                    "}" _ {
                    IrAstFnDecl {
                        name,
                        args,
                        ret_type,
                        is_public: true,
                        metadata,
                        locals,
                        blocks,
                        selector
                    }
                }
                / "fn" _ name:id() _ selector:selector_id()? _ "(" _
                        args:(block_arg() ** comma()) ")" _ "->" _ ret_type:ast_ty()
                            metadata:comma_metadata_idx()? "{" _
                        locals:fn_local()*
                        blocks:block_decl()*
                    "}" _ {
                    IrAstFnDecl {
                        name,
                        args,
                        ret_type,
                        is_public: false,
                        metadata,
                        locals,
                        blocks,
                        selector
                    }
                }

            rule selector_id() -> [u8; 4]
                = "<" _ s:$(['0'..='9' | 'a'..='f' | 'A'..='F']*<8>) _ ">" _ {
                    string_to_hex::<4>(s)
                }

            rule block_arg() -> (IrAstTy, String, Option<MdIdxRef>)
                = name:id() mdi:metadata_idx()? ":" _ ty:ast_ty() {
                    (ty, name, mdi)
                }

            rule fn_local() -> (IrAstTy, String, bool, Option<IrAstOperation>)
                = "local" _ im:mut_ptr() ty:ast_ty() name:id() init:fn_local_init()? {
                    (ty, name, im, init)
                }

            rule fn_local_init() -> IrAstOperation
                = "=" _ cv:op_const() {
                    cv
                }

            rule block_decl() -> IrAstBlock
                = label:id() "(" _ args:(block_arg() ** comma()) ")" _
                    ":" _ instructions: instr_decl()* {
                    IrAstBlock {
                        label,
                        args,
                        instructions
                    }
                }

            rule instr_decl() -> IrAstInstruction
                = value_name:value_assign()? op:operation() metadata:comma_metadata_idx()? {
                    IrAstInstruction {
                        value_name,
                        op,
                        metadata,
                    }
                }

            rule value_assign() -> String
                = name:id() "=" _ {
                    name
                }

            rule metadata_idx() -> MdIdxRef
                = "!" idx:decimal() {
                    idx
                }

            rule comma_metadata_idx() -> MdIdxRef
                = "," _ mdi:metadata_idx() {
                    mdi
                }

            rule binary_op_kind() -> BinaryOpKind
                = "add" _ { BinaryOpKind::Add }
                / "sub" _ { BinaryOpKind::Sub }
                / "mul" _ { BinaryOpKind::Mul }
                / "div" _ { BinaryOpKind::Div }

            rule operation() -> IrAstOperation
                = op_addr_of()
                / op_asm()
                / op_branch()
                / op_bitcast()
                / op_binary()
                / op_call()
                / op_cbr()
                / op_cmp()
                / op_const()
                / op_contract_call()
                / op_extract_element()
                / op_extract_value()
                / op_get_storage_key()
                / op_get_ptr()
                / op_gtf()
                / op_insert_element()
                / op_insert_value()
                / op_int_to_ptr()
                / op_load()
                / op_log()
                / op_mem_copy()
                / op_nop()
                / op_read_register()
                / op_ret()
                / op_revert()
                / op_state_load_quad_word()
                / op_state_load_word()
                / op_state_store_quad_word()
                / op_state_store_word()
                / op_store()

            rule op_addr_of() -> IrAstOperation
                = "addr_of" _ val:id() {
                    IrAstOperation::AddrOf(val)
                }

            rule op_asm() -> IrAstOperation
                = "asm" _ "(" _ args:(asm_arg() ** comma()) ")" _ ret:asm_ret()? meta_idx:comma_metadata_idx()? "{" _
                    ops:asm_op()*
                "}" _ {
                    IrAstOperation::Asm(
                        args,
                        ret.clone().map(|(ty, _)| ty).unwrap_or(IrAstTy::Unit),
                        ret.map(|(_, nm)| nm),
                        ops,
                        meta_idx
                    )
                }

            rule op_bitcast() -> IrAstOperation
                = "bitcast" _ val:id() "to" _ ty:ast_ty() {
                    IrAstOperation::BitCast(val, ty)
                }

            rule op_binary() -> IrAstOperation
                = op: binary_op_kind() arg1:id() comma() arg2:id() {
                    IrAstOperation::BinaryOp(op, arg1, arg2)
                }

            rule op_branch() -> IrAstOperation
                = "br" _ to_block:id() "(" _ args:(id() ** comma()) ")" _ {
                    IrAstOperation::Br(to_block, args)
                }

            rule op_call() -> IrAstOperation
                = "call" _ callee:id() "(" _ args:(id() ** comma()) ")" _ {
                    IrAstOperation::Call(callee, args)
            }

            rule op_cbr() -> IrAstOperation
                = "cbr" _ cond:id() comma() tblock:id()
                "(" _ targs:(id() ** comma()) ")" _
                 comma() fblock:id() "(" _ fargs:(id() ** comma()) ")" _ {
                    IrAstOperation::Cbr(cond, tblock, targs, fblock, fargs)
                }

            rule op_cmp() -> IrAstOperation
                = "cmp" _ p:cmp_pred() l:id() r:id() {
                    IrAstOperation::Cmp(p, l, r)
                }

            rule op_const() -> IrAstOperation
                = "const" _ val_ty:ast_ty() cv:constant() {
                    IrAstOperation::Const(val_ty, cv)
                }

            rule op_contract_call() -> IrAstOperation
                = "contract_call" _
                ty:ast_ty() _ name:id() _
                params:id() comma() coins:id() comma() asset_id:id() comma() gas:id() _ {
                    IrAstOperation::ContractCall(ty, name, params, coins, asset_id, gas)
            }

            rule op_extract_element() -> IrAstOperation
                = "extract_element" _ name:id() comma() ty:ast_ty() comma() idx:id() {
                    IrAstOperation::ExtractElement(name, ty, idx)
                }

            rule op_extract_value() -> IrAstOperation
                = "extract_value" _ name:id() comma() ty:ast_ty() comma() idcs:(decimal() ++ comma()) {
                    IrAstOperation::ExtractValue(name, ty, idcs)
                }

            rule op_get_storage_key() -> IrAstOperation
                = "get_storage_key" _ {
                    IrAstOperation::GetStorageKey()
                }

            rule op_get_ptr() -> IrAstOperation
                = "get_ptr" _ mut_ptr() ty:ast_ty() name:id()
                    comma() ptr() ty:ast_ty() comma() offset:(decimal())  {
                    IrAstOperation::GetPtr(name, ty, offset)
                }

            rule op_gtf() -> IrAstOperation
                = "gtf" _ index:id() comma() tx_field_id:decimal()  {
                    IrAstOperation::Gtf(index, tx_field_id)
                }

            rule op_insert_element() -> IrAstOperation
                = "insert_element" _ name:id() comma() ty:ast_ty() comma() val:id() comma() idx:id() {
                    IrAstOperation::InsertElement(name, ty, val, idx)
                }

            rule op_insert_value() -> IrAstOperation
                = "insert_value" _ aval:id() comma() ty:ast_ty() comma() ival:id() comma() idcs:(decimal() ++ comma()) {
                    IrAstOperation::InsertValue(aval, ty, ival, idcs)
                }

            rule op_int_to_ptr() -> IrAstOperation
                = "int_to_ptr" _ val:id() "to" _ ty:ast_ty() {
                    IrAstOperation::IntToPtr(val, ty)
                }

            rule op_load() -> IrAstOperation
                = "load" _ ptr() src:id() {
                    IrAstOperation::Load(src)
                }

            rule op_log() -> IrAstOperation
                = "log" _ log_ty:ast_ty() log_val:id() comma() log_id:id() {
                    IrAstOperation::Log(log_ty, log_val, log_id)
                }

            rule op_mem_copy() -> IrAstOperation
                = "mem_copy" _ dst_name:id() comma() src_name:id() comma() len:decimal() {
                    IrAstOperation::MemCopy(dst_name, src_name, len)
                }

            rule op_nop() -> IrAstOperation
                = "nop" _ {
                    IrAstOperation::Nop
                }

            rule op_read_register() -> IrAstOperation
                = "read_register" _ r:reg_name() {
                    IrAstOperation::ReadRegister(r)
                }

            rule op_ret() -> IrAstOperation
                = "ret" _ ty:ast_ty() vn:id() {
                    IrAstOperation::Ret(ty, vn)
                }

            rule op_revert() -> IrAstOperation
                = "revert" _ vn:id() {
                    IrAstOperation::Revert(vn)
                }

            rule op_state_load_quad_word() -> IrAstOperation
                = "state_load_quad_word" _ ptr() dst:id() comma() "key" _ ptr() _ key:id() {
                    IrAstOperation::StateLoadQuadWord(dst, key)
                }

            rule op_state_load_word() -> IrAstOperation
                = "state_load_word" _ "key" _ ptr() _ key:id() {
                    IrAstOperation::StateLoadWord(key)
                }

            rule op_state_store_quad_word() -> IrAstOperation
                = "state_store_quad_word" _ ptr() src:id() comma() "key" _ ptr() _ key:id() {
                    IrAstOperation::StateStoreQuadWord(src, key)
                }

            rule op_state_store_word() -> IrAstOperation
                = "state_store_word" _ src:id() comma() "key" _ ptr() _ key:id() {
                    IrAstOperation::StateStoreWord(src, key)
                }

            rule op_store() -> IrAstOperation
                = "store" _ val:id() comma() ptr() dst:id() {
                    IrAstOperation::Store(val, dst)
                }

            rule cmp_pred() -> String
                = p:$("eq") _ {
                    p.to_string()
                }

            rule reg_name() -> String
                = r:$("of" / "pc" / "ssp" / "sp" / "fp" / "hp" / "err" / "ggas" / "cgas" / "bal" / "is" / "ret" / "retl" / "flag") _ {
                    r.to_string()
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

            rule asm_ret() -> (IrAstTy, Ident)
                = "->" _ ty:ast_ty() ret:id_id() {
                    (ty, ret)
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
                    Ident::new(Span::new(imm.into(), 0, imm.len(), None).unwrap())
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
                / "0x" s:$(hex_digit()*<64>) _ {
                    IrAstConstValue::B256(string_to_hex::<32>(s))
                }
                / n:decimal() { IrAstConstValue::Number(n) }
                / string_const()
                / array_const()
                / struct_const()

            rule string_const() -> IrAstConstValue
                = ['"'] chs:str_char()* ['"'] _ {
                    IrAstConstValue::String(chs)
                }

            rule str_char() -> u8
                // Match any of the printable characters except '"' and '\'.
                = c:$([' ' | '!' | '#'..='[' | ']'..='~']) {
                    *c.as_bytes().first().unwrap()
                }
                / "\\x" h:hex_digit() l:hex_digit() {
                    (h << 4) | l
                }

            //  There may be a better way to do this, dunno.  In `str_char()` we're parsing '\xHH'
            //  from a hex byte to a u8.  We do it by parsing each hex nybble into a u8 and then OR
            //  them together.  In hex_digit(), to convert e.g., 'c' to 12, we match the pattern,
            //  convert the str into a u8 iterator, take the first value which is the ascii digit,
            //  convert the 'A'-'F' to uppercase by setting the 6th bit (0x20) and subtracting the
            //  right offset.  Fiddly.
            rule hex_digit() -> u8
                = d:$(['0'..='9']) {
                    d.as_bytes().first().unwrap() - b'0'
                }
                / d:$(['a'..='f' | 'A'..='F']) {
                    (d.as_bytes().first().unwrap() | 0x20) - b'a' + 10
                }

            rule array_const() -> IrAstConstValue
                = "[" _ els:(field_or_element_const() ++ comma()) "]" _ {
                    let el_ty = els[0].0.clone();
                    let els = els.into_iter().map(|(_, cv)| cv).collect::<Vec<_>>();
                    IrAstConstValue::Array(el_ty, els)
                }

            rule struct_const() -> IrAstConstValue
                = "{" _ flds:(field_or_element_const() ** comma()) "}" _ {
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
                / struct_ty()
                / union_ty()
                / mp:mut_ptr() ty:ast_ty() { IrAstTy::Pointer(Box::new(ty), mp) }

            rule array_ty() -> IrAstTy
                = "[" _ ty:ast_ty() ";" _ c:decimal() "]" _ {
                    IrAstTy::Array(Box::new(ty), c)
                }

            rule union_ty() -> IrAstTy
                = "(" _ tys:(ast_ty() ++ ("|" _)) ")" _ {
                    IrAstTy::Union(tys)
                }

            rule struct_ty() -> IrAstTy
                = "{" _ tys:(ast_ty() ** comma()) "}" _ {
                    IrAstTy::Struct(tys)
                }

            rule id() -> String
                = !ast_ty() id:$(id_char0() id_char()*) _ {
                    id.to_owned()
                }

            rule id_id() -> Ident
                = !ast_ty() id:$(id_char0() id_char()*) _ {
                    Ident::new(Span::new(id.into(), 0, id.len(), None).unwrap())
                }

            // Metadata decls are sensitive to the newlines since the assignee idx could belong to
            // the previous decl otherwise.  e.g.,
            //
            //   !1 = blah !2
            //   !2 = 42
            //
            // If we did not make newlines significant we could parse the first struct as
            // `!1 = blah !2 !2` and then get an error on the following `=`.
            //
            // An alternative is to put some other delimiter around naked indices, but using
            // newlines below hasn't been that painful, so that'll do for now.

            rule metadata_decls() -> Vec<(MdIdxRef, IrMetadatum)>
                = ds:(metadata_decl() ** nl()) _ {
                    ds
                }

            rule metadata_decl() -> (MdIdxRef, IrMetadatum)
                = idx:metadata_idx() "=" _ item:metadata_item() {
                    (idx, item)
                }

            // This rule (uniquely) does NOT discard the newline whitespace. `__` matches only
            // spaces.
            rule metadata_item() -> IrMetadatum
                = i:dec_digits() __ {
                    IrMetadatum::Integer(i)
                }
                / "!" idx:dec_digits() __ {
                    IrMetadatum::Index(idx)
                }
                / ['"'] s:$(([^ '"' | '\\'] / ['\\'] ['\\' | '"' ])+) ['"'] __ {
                    IrMetadatum::String(s.to_owned())
                }
                / tag:$(id_char0() id_char()*) __ els:metadata_item()* {
                    IrMetadatum::Struct(tag.to_owned(), els)
                }
                / "(" _ els:metadata_idx()*<2,> ")" __ {
                    // Lists must contain at least 2 items, otherwise they needn't be lists.
                    IrMetadatum::List(els)
                }

            rule id_char0()
                = quiet!{ ['A'..='Z' | 'a'..='z' | '_'] }

            rule id_char()
                = quiet!{ id_char0() / ['0'..='9'] }

            rule decimal() -> u64
                = d:dec_digits() _ {
                    d
                }

            // String of decimal digits without discarding whitespace. (Useful for newline
            // sensitive metadata).
            rule dec_digits() -> u64
                = ds:$("0" / ['1'..='9'] ['0'..='9']*) {
                    ds.parse::<u64>().unwrap()
                }

            rule ptr()
                = "ptr" _

            rule mut_ptr() -> bool
                = m:("mut" _)? ptr() {
                    m.is_some()
                }

            rule comma()
                = quiet!{ "," _ }

            rule _()
                = quiet!{ (space() / nl() / comment())* }

            rule __()
                = quiet!{ (space() / comment())* }

            rule space()
                = [' ' | '\t']

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
        constant::{Constant, ConstantValue},
        context::Context,
        error::IrError,
        function::Function,
        instruction::{Instruction, Predicate, Register},
        irtype::{Aggregate, Type},
        metadata::{MetadataIndex, Metadatum},
        module::{Kind, Module},
        pointer::Pointer,
        value::Value,
        BinaryOpKind, BlockArgument,
    };

    #[derive(Debug)]
    pub(super) struct IrAstModule {
        kind:     Kind,
        fn_decls: Vec<IrAstFnDecl>,
        metadata: Vec<(MdIdxRef, IrMetadatum)>,
    }

    #[derive(Debug)]
    struct IrAstFnDecl {
        name:      String,
        args:      Vec<(IrAstTy, String, Option<MdIdxRef>)>,
        ret_type:  IrAstTy,
        is_public: bool,
        metadata:  Option<MdIdxRef>,
        locals:    Vec<(IrAstTy, String, bool, Option<IrAstOperation>)>,
        blocks:    Vec<IrAstBlock>,
        selector:  Option<[u8; 4]>,
    }

    #[derive(Debug)]
    struct IrAstBlock {
        label:        String,
        args:         Vec<(IrAstTy, String, Option<MdIdxRef>)>,
        instructions: Vec<IrAstInstruction>,
    }

    #[derive(Debug)]
    struct IrAstInstruction {
        value_name: Option<String>,
        op:         IrAstOperation,
        metadata:   Option<MdIdxRef>,
    }

    #[derive(Debug)]
    enum IrAstOperation {
        AddrOf(String),
        Asm(
            Vec<(Ident, Option<IrAstAsmArgInit>)>,
            IrAstTy,
            Option<Ident>,
            Vec<IrAstAsmOp>,
            Option<MdIdxRef>,
        ),
        BitCast(String, IrAstTy),
        BinaryOp(BinaryOpKind, String, String),
        Br(String, Vec<String>),
        Call(String, Vec<String>),
        Cbr(String, String, Vec<String>, String, Vec<String>),
        Cmp(String, String, String),
        Const(IrAstTy, IrAstConst),
        ContractCall(IrAstTy, String, String, String, String, String),
        ExtractElement(String, IrAstTy, String),
        ExtractValue(String, IrAstTy, Vec<u64>),
        GetStorageKey(),
        GetPtr(String, IrAstTy, u64),
        Gtf(String, u64),
        InsertElement(String, IrAstTy, String, String),
        InsertValue(String, IrAstTy, String, Vec<u64>),
        IntToPtr(String, IrAstTy),
        Load(String),
        Log(IrAstTy, String, String),
        MemCopy(String, String, u64),
        Nop,
        ReadRegister(String),
        Ret(IrAstTy, String),
        Revert(String),
        StateLoadQuadWord(String, String),
        StateLoadWord(String),
        StateStoreQuadWord(String, String),
        StateStoreWord(String, String),
        Store(String, String),
    }

    #[derive(Debug)]
    struct IrAstConst {
        value:    IrAstConstValue,
        meta_idx: Option<MdIdxRef>,
    }

    #[derive(Debug)]
    enum IrAstConstValue {
        Undef(IrAstTy),
        Unit,
        Bool(bool),
        B256([u8; 32]),
        Number(u64),
        String(Vec<u8>),
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
        name:     Ident,
        args:     Vec<Ident>,
        imm:      Option<Ident>,
        meta_idx: Option<MdIdxRef>,
    }

    impl IrAstConstValue {
        fn as_constant_value(&self, context: &mut Context) -> ConstantValue {
            match self {
                IrAstConstValue::Undef(_) => ConstantValue::Undef,
                IrAstConstValue::Unit => ConstantValue::Unit,
                IrAstConstValue::Bool(b) => ConstantValue::Bool(*b),
                IrAstConstValue::B256(bs) => ConstantValue::B256(*bs),
                IrAstConstValue::Number(n) => ConstantValue::Uint(*n),
                IrAstConstValue::String(bs) => ConstantValue::String(bs.clone()),
                IrAstConstValue::Array(el_ty, els) => {
                    let els: Vec<_> = els
                        .iter()
                        .map(|cv| cv.value.as_constant(context, el_ty.clone()))
                        .collect();
                    ConstantValue::Array(els)
                }
                IrAstConstValue::Struct(flds) => {
                    let fields: Vec<_> = flds
                        .iter()
                        .map(|(ty, cv)| cv.value.as_constant(context, ty.clone()))
                        .collect::<Vec<_>>();
                    ConstantValue::Struct(fields)
                }
            }
        }

        fn as_constant(&self, context: &mut Context, val_ty: IrAstTy) -> Constant {
            Constant {
                ty:    val_ty.to_ir_type(context),
                value: self.as_constant_value(context),
            }
        }

        fn as_value(&self, context: &mut Context, val_ty: IrAstTy) -> Value {
            match self {
                IrAstConstValue::Undef(_) => unreachable!("Can't convert 'undef' to a value."),
                IrAstConstValue::Unit => Constant::get_unit(context),
                IrAstConstValue::Bool(b) => Constant::get_bool(context, *b),
                IrAstConstValue::B256(bs) => Constant::get_b256(context, *bs),
                IrAstConstValue::Number(n) => Constant::get_uint(context, 64, *n),
                IrAstConstValue::String(s) => Constant::get_string(context, s.clone()),
                IrAstConstValue::Array(..) => {
                    let array_const = self.as_constant(context, val_ty);
                    Constant::get_array(context, array_const)
                }
                IrAstConstValue::Struct(_) => {
                    let struct_const = self.as_constant(context, val_ty);
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
        Pointer(Box<IrAstTy>, bool),
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
                IrAstTy::Pointer(ty, is_mut) => {
                    let ty = ty.to_ir_type(context);
                    Type::Pointer(Pointer::new(context, ty, *is_mut, None))
                }
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
        /// A number.
        Integer(u64),
        /// A reference to another metadatum.
        Index(MdIdxRef),
        /// An arbitrary string (e.g., a path).
        String(String),
        /// A tagged collection of metadata (e.g., `span !1 10 20`).
        Struct(String, Vec<IrMetadatum>),
        /// A collection of indices to other metadata, for attaching multiple metadata to values.
        List(Vec<MdIdxRef>),
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

    use std::{collections::HashMap, iter::FromIterator};

    pub(super) fn build_context(ir_ast_mod: IrAstModule) -> Result<Context, IrError> {
        let mut ctx = Context::default();
        let mut builder = IrBuilder {
            module:           Module::new(&mut ctx, ir_ast_mod.kind),
            md_map:           build_metadata_map(&mut ctx, ir_ast_mod.metadata),
            unresolved_calls: Vec::new(),
        };

        for fn_decl in ir_ast_mod.fn_decls {
            builder.add_fn_decl(&mut ctx, fn_decl)?;
        }
        builder.resolve_calls(&mut ctx)?;

        Ok(ctx)
    }

    struct IrBuilder {
        module:           Module,
        md_map:           HashMap<MdIdxRef, MetadataIndex>,
        unresolved_calls: Vec<PendingCall>,
    }

    struct PendingCall {
        call_val: Value,
        callee:   String,
    }

    impl IrBuilder {
        fn add_fn_decl(
            &mut self,
            context: &mut Context,
            fn_decl: IrAstFnDecl,
        ) -> Result<(), IrError> {
            let convert_md_idx = |opt_md_idx: &Option<MdIdxRef>| {
                opt_md_idx.map(|mdi| self.md_map.get(&mdi).copied().unwrap())
            };
            let args: Vec<(String, Type, Option<MetadataIndex>)> = fn_decl
                .args
                .iter()
                .map(|(ty, name, md_idx)| {
                    (name.into(), ty.to_ir_type(context), convert_md_idx(md_idx))
                })
                .collect();
            let ret_type = fn_decl.ret_type.to_ir_type(context);
            let func = Function::new(
                context,
                self.module,
                fn_decl.name,
                args,
                ret_type,
                fn_decl.selector,
                fn_decl.is_public,
                convert_md_idx(&fn_decl.metadata),
            );

            // Gather all the (new) arg values by name into a map.
            let mut arg_map = HashMap::<String, Value>::new();
            let mut ptr_map = HashMap::<String, Pointer>::new();
            for (ty, name, is_mutable, initializer) in fn_decl.locals {
                let initializer = initializer.map(|const_init| {
                    if let IrAstOperation::Const(val_ty, val) = const_init {
                        val.value.as_constant(context, val_ty)
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

            // The entry block is already created, we don't want to recreate it.
            let named_blocks = HashMap::from_iter(fn_decl.blocks.iter().map(|block| {
                (
                    block.label.clone(),
                    if block.label == "entry" {
                        func.get_entry_block(context)
                    } else {
                        let irblock = func.create_block(context, Some(block.label.clone()));
                        for (idx, arg) in block.args.iter().enumerate() {
                            let ty = arg.0.to_ir_type(context);
                            let arg = Value::new_argument(
                                context,
                                BlockArgument {
                                    block: irblock,
                                    idx,
                                    ty,
                                },
                            )
                            .add_metadatum(context, convert_md_idx(&arg.2));
                            irblock.add_arg(context, arg);
                        }
                        irblock
                    },
                )
            }));

            for block in fn_decl.blocks {
                for (idx, arg) in block.args.iter().enumerate() {
                    arg_map.insert(
                        arg.1.clone(),
                        named_blocks[&block.label].get_arg(context, idx).unwrap(),
                    );
                }
                self.add_block_instructions(context, block, &named_blocks, &ptr_map, &mut arg_map);
            }
            Ok(())
        }

        fn add_block_instructions(
            &mut self,
            context: &mut Context,
            ir_block: IrAstBlock,
            named_blocks: &HashMap<String, Block>,
            ptr_map: &HashMap<String, Pointer>,
            val_map: &mut HashMap<String, Value>,
        ) {
            let block = named_blocks.get(&ir_block.label).unwrap();
            for ins in ir_block.instructions {
                let opt_metadata = ins
                    .metadata
                    .map(|mdi| self.md_map.get(&mdi).unwrap())
                    .copied();
                let ins_val = match ins.op {
                    IrAstOperation::AddrOf(val) => block
                        .ins(context)
                        .addr_of(*val_map.get(&val).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Asm(args, return_type, return_name, ops, meta_idx) => {
                        let args = args
                            .into_iter()
                            .map(|(name, opt_init)| AsmArg {
                                name,
                                initializer: opt_init.map(|init| match init {
                                    IrAstAsmArgInit::Var(var) => {
                                        val_map.get(&var).cloned().unwrap()
                                    }
                                    IrAstAsmArgInit::Imm(cv) => {
                                        cv.value.as_value(context, IrAstTy::U64).add_metadatum(
                                            context,
                                            self.md_map.get(cv.meta_idx.as_ref().unwrap()).copied(),
                                        )
                                    }
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
                                    metadata: meta_idx
                                        .as_ref()
                                        .and_then(|meta_idx| self.md_map.get(meta_idx).copied()),
                                },
                            )
                            .collect();
                        let md_idx = meta_idx.map(|mdi| self.md_map.get(&mdi).unwrap()).copied();
                        let return_type = return_type.to_ir_type(context);
                        block
                            .ins(context)
                            .asm_block(args, body, return_type, return_name)
                            .add_metadatum(context, md_idx)
                    }
                    IrAstOperation::BitCast(val, ty) => {
                        let to_ty = ty.to_ir_type(context);
                        block
                            .ins(context)
                            .bitcast(*val_map.get(&val).unwrap(), to_ty)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::BinaryOp(op, arg1, arg2) => block
                        .ins(context)
                        .binary_op(
                            op,
                            *val_map.get(&arg1).unwrap(),
                            *val_map.get(&arg2).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Br(to_block_name, args) => {
                        let to_block = named_blocks.get(&to_block_name).unwrap();
                        block
                            .ins(context)
                            .branch(
                                *to_block,
                                args.iter().map(|arg| *val_map.get(arg).unwrap()).collect(),
                            )
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::Call(callee, args) => {
                        // We can't resolve calls to other functions until we've done a first pass and
                        // created them first.  So we can insert a dummy call here, save the call
                        // params and update it with the proper callee function in a second pass.
                        //
                        // The dummy function we'll use for now is just the current function.
                        let dummy_func = block.get_function(context);
                        let call_val = block
                            .ins(context)
                            .call(
                                dummy_func,
                                &args
                                    .iter()
                                    .map(|arg_name| val_map.get(arg_name).unwrap())
                                    .cloned()
                                    .collect::<Vec<Value>>(),
                            )
                            .add_metadatum(context, opt_metadata);
                        self.unresolved_calls.push(PendingCall { call_val, callee });
                        call_val
                    }
                    IrAstOperation::Cbr(
                        cond_val_name,
                        true_block_name,
                        true_args,
                        false_block_name,
                        false_args,
                    ) => block
                        .ins(context)
                        .conditional_branch(
                            *val_map.get(&cond_val_name).unwrap(),
                            *named_blocks.get(&true_block_name).unwrap(),
                            *named_blocks.get(&false_block_name).unwrap(),
                            true_args
                                .iter()
                                .map(|arg| *val_map.get(arg).unwrap())
                                .collect(),
                            false_args
                                .iter()
                                .map(|arg| *val_map.get(arg).unwrap())
                                .collect(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Cmp(pred_str, lhs, rhs) => block
                        .ins(context)
                        .cmp(
                            match pred_str.as_str() {
                                "eq" => Predicate::Equal,
                                _ => unreachable!("Bug in `cmp` predicate rule."),
                            },
                            *val_map.get(&lhs).unwrap(),
                            *val_map.get(&rhs).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Const(ty, val) => val
                        .value
                        .as_value(context, ty)
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::ContractCall(
                        return_type,
                        name,
                        params,
                        coins,
                        asset_id,
                        gas,
                    ) => {
                        let ir_ty = return_type.to_ir_type(context);
                        block
                            .ins(context)
                            .contract_call(
                                ir_ty,
                                name,
                                *val_map.get(&params).unwrap(),
                                *val_map.get(&coins).unwrap(),
                                *val_map.get(&asset_id).unwrap(),
                                *val_map.get(&gas).unwrap(),
                            )
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::ExtractElement(aval, ty, idx) => {
                        let ir_ty = ty.to_ir_aggregate_type(context);
                        block
                            .ins(context)
                            .extract_element(
                                *val_map.get(&aval).unwrap(),
                                ir_ty,
                                *val_map.get(&idx).unwrap(),
                            )
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::ExtractValue(val, ty, idcs) => {
                        let ir_ty = ty.to_ir_aggregate_type(context);
                        block
                            .ins(context)
                            .extract_value(*val_map.get(&val).unwrap(), ir_ty, idcs)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::GetStorageKey() => block
                        .ins(context)
                        .get_storage_key()
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::GetPtr(base_ptr, ptr_ty, offset) => {
                        let ptr_ir_ty = ptr_ty.to_ir_type(context);
                        block
                            .ins(context)
                            .get_ptr(*ptr_map.get(&base_ptr).unwrap(), ptr_ir_ty, offset)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::Gtf(index, tx_field_id) => block
                        .ins(context)
                        .gtf(*val_map.get(&index).unwrap(), tx_field_id)
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::InsertElement(aval, ty, val, idx) => {
                        let ir_ty = ty.to_ir_aggregate_type(context);
                        block
                            .ins(context)
                            .insert_element(
                                *val_map.get(&aval).unwrap(),
                                ir_ty,
                                *val_map.get(&val).unwrap(),
                                *val_map.get(&idx).unwrap(),
                            )
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::InsertValue(aval, ty, ival, idcs) => {
                        let ir_ty = ty.to_ir_aggregate_type(context);
                        block
                            .ins(context)
                            .insert_value(
                                *val_map.get(&aval).unwrap(),
                                ir_ty,
                                *val_map.get(&ival).unwrap(),
                                idcs,
                            )
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::IntToPtr(val, ty) => {
                        let to_ty = ty.to_ir_type(context);
                        block
                            .ins(context)
                            .int_to_ptr(*val_map.get(&val).unwrap(), to_ty)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::Load(src_name) => block
                        .ins(context)
                        .load(*val_map.get(&src_name).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Log(log_ty, log_val, log_id) => {
                        let log_ty = log_ty.to_ir_type(context);
                        block
                            .ins(context)
                            .log(
                                *val_map.get(&log_val).unwrap(),
                                log_ty,
                                *val_map.get(&log_id).unwrap(),
                            )
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::MemCopy(dst_name, src_name, len) => block
                        .ins(context)
                        .mem_copy(
                            *val_map.get(&dst_name).unwrap(),
                            *val_map.get(&src_name).unwrap(),
                            len,
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Nop => block.ins(context).nop(),
                    IrAstOperation::ReadRegister(reg_name) => block
                        .ins(context)
                        .read_register(match reg_name.as_str() {
                            "of" => Register::Of,
                            "pc" => Register::Pc,
                            "ssp" => Register::Ssp,
                            "sp" => Register::Sp,
                            "fp" => Register::Fp,
                            "hp" => Register::Hp,
                            "err" => Register::Error,
                            "ggas" => Register::Ggas,
                            "cgas" => Register::Cgas,
                            "bal" => Register::Bal,
                            "is" => Register::Is,
                            "ret" => Register::Ret,
                            "retl" => Register::Retl,
                            "flag" => Register::Flag,
                            _ => unreachable!("Guaranteed by grammar."),
                        })
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Ret(ty, ret_val_name) => {
                        let ty = ty.to_ir_type(context);
                        block
                            .ins(context)
                            .ret(*val_map.get(&ret_val_name).unwrap(), ty)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::Revert(ret_val_name) => block
                        .ins(context)
                        .revert(*val_map.get(&ret_val_name).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::StateLoadQuadWord(dst, key) => block
                        .ins(context)
                        .state_load_quad_word(
                            *val_map.get(&dst).unwrap(),
                            *val_map.get(&key).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::StateLoadWord(key) => block
                        .ins(context)
                        .state_load_word(*val_map.get(&key).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::StateStoreQuadWord(src, key) => block
                        .ins(context)
                        .state_store_quad_word(
                            *val_map.get(&src).unwrap(),
                            *val_map.get(&key).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::StateStoreWord(src, key) => block
                        .ins(context)
                        .state_store_word(*val_map.get(&src).unwrap(), *val_map.get(&key).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Store(stored_val_name, dst_val_name) => block
                        .ins(context)
                        .store(
                            *val_map.get(&dst_val_name).unwrap(),
                            *val_map.get(&stored_val_name).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                };
                ins.value_name.map(|vn| val_map.insert(vn, ins_val));
            }
        }

        fn resolve_calls(self, context: &mut Context) -> Result<(), IrError> {
            // All of the call instructions are currently invalid (recursive) CALLs to their own
            // function, which need to be replaced with the proper callee function.  We couldn't do
            // it above until we'd gone and created all the functions first.
            //
            // Now we can loop and find the callee function for each call and update them.
            for pending_call in self.unresolved_calls {
                let call_func = context
                    .functions
                    .iter()
                    .find_map(|(idx, content)| {
                        if content.name == pending_call.callee {
                            Some(Function(idx))
                        } else {
                            None
                        }
                    })
                    .unwrap();

                if let Some(Instruction::Call(dummy_func, _args)) =
                    pending_call.call_val.get_instruction_mut(context)
                {
                    *dummy_func = call_func;
                }
            }
            Ok(())
        }
    }

    /// Create the metadata for the module in `context` and generate a map from the parsed
    /// `MdIdxRef`s to the new actual metadata.
    fn build_metadata_map(
        context: &mut Context,
        ir_metadata: Vec<(MdIdxRef, IrMetadatum)>,
    ) -> HashMap<MdIdxRef, MetadataIndex> {
        fn convert_md(
            context: &mut Context,
            md: IrMetadatum,
            md_map: &mut HashMap<MdIdxRef, MetadataIndex>,
        ) -> Metadatum {
            match md {
                IrMetadatum::Integer(i) => Metadatum::Integer(i),
                IrMetadatum::Index(idx) => Metadatum::Index(
                    md_map
                        .get(&idx)
                        .copied()
                        .expect("Metadatum index not found in map."),
                ),
                IrMetadatum::String(s) => Metadatum::String(s),
                IrMetadatum::Struct(tag, els) => Metadatum::Struct(
                    tag,
                    els.into_iter()
                        .map(|el_md| convert_md(context, el_md, md_map))
                        .collect(),
                ),
                IrMetadatum::List(idcs) => Metadatum::List(
                    idcs.into_iter()
                        .map(|idx| {
                            md_map
                                .get(&idx)
                                .copied()
                                .expect("Metadatum index not found in map.")
                        })
                        .collect(),
                ),
            }
        }

        let mut md_map = HashMap::new();

        for (ir_idx, ir_md) in ir_metadata {
            let md = convert_md(context, ir_md, &mut md_map);
            let md_idx = MetadataIndex(context.metadata.insert(md));
            md_map.insert(ir_idx, md_idx);
        }
        md_map
    }

    fn string_to_hex<const N: usize>(s: &str) -> [u8; N] {
        let mut bytes: [u8; N] = [0; N];
        let mut cur_byte: u8 = 0;
        for (idx, ch) in s.chars().enumerate() {
            cur_byte = (cur_byte << 4) | ch.to_digit(16).unwrap() as u8;
            if idx % 2 == 1 {
                bytes[idx / 2] = cur_byte;
                cur_byte = 0;
            }
        }
        bytes
    }
}

// -------------------------------------------------------------------------------------------------
