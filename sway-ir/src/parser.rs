//! A parser for the printed IR, useful mostly for testing.

use sway_features::ExperimentalFeatures;
use sway_types::SourceEngine;

use crate::{context::Context, error::IrError, Backtrace};

// -------------------------------------------------------------------------------------------------
/// Parse a string produced by [`crate::printer::to_string`] into a new [`Context`].
pub fn parse<'eng>(
    input: &str,
    source_engine: &'eng SourceEngine,
    experimental: ExperimentalFeatures,
    backtrace: Backtrace,
) -> Result<Context<'eng>, IrError> {
    let irmod = ir_builder::parser::ir_descrs(input).map_err(|err| {
        let found = if input.len() - err.location.offset <= 20 {
            &input[err.location.offset..]
        } else {
            &input[err.location.offset..][..20]
        };
        IrError::ParseFailure(err.to_string(), found.into())
    })?;
    let ir = ir_builder::build_context(irmod, source_engine, experimental, backtrace)?;
    ir.verify()?;
    Ok(ir)
}

// -------------------------------------------------------------------------------------------------

mod ir_builder {
    use slotmap::KeyData;
    use std::convert::TryFrom;
    use sway_features::ExperimentalFeatures;
    use sway_types::{ident::Ident, span::Span, u256::U256, SourceEngine};

    type MdIdxRef = u64;

    peg::parser! {
        pub(in crate::parser) grammar parser() for str {
            pub(in crate::parser) rule ir_descrs() -> IrAstModule
                = _ sop:script_or_predicate() eoi() {
                    sop
                }
                / _ c:contract() eoi() {
                    c
                }

            rule script_or_predicate() -> IrAstModule
                = kind:module_kind() "{" _ configs:init_config()* _ global_vars:global_var()* _ fn_decls:fn_decl()* "}" _
                  metadata:metadata_decls() {
                    IrAstModule {
                        kind,
                        configs,
                        global_vars,
                        storage_keys: vec![],
                        fn_decls,
                        metadata
                    }
                }

            rule module_kind() -> Kind
                = "script" _ { Kind::Script }
                / "predicate" _ { Kind::Predicate }

            rule contract() -> IrAstModule
                = "contract" _ "{" _
                  configs:init_config()* _ global_vars:global_var()* _ storage_keys:storage_key()* _ fn_decls:fn_decl()* "}" _
                  metadata:metadata_decls() {
                    IrAstModule {
                        kind: crate::module::Kind::Contract,
                        configs,
                        global_vars,
                        storage_keys,
                        fn_decls,
                        metadata
                    }
                }

            rule global_var() -> IrAstGlobalVar
                = "global" _ m:("mut" _)? name:path() _ ":" _ ty:ast_ty() init:global_init()? {
                    IrAstGlobalVar {
                        name,
                        ty,
                        init,
                        mutable: m.is_some(),
                    }
                }

            rule global_init() -> IrAstOperation
                = "=" _ cv:op_const() {
                    cv
                }

            rule config_encoded_bytes() -> Vec<u8>
                = "0x" s:$(hex_digit()*) _ {
                    hex_string_to_vec(s)
                }

            rule init_config() -> IrAstConfig
                = value_name:value_assign() "config" _ val_ty:ast_ty() _ "," _ decode_fn:id() _ "," _ encoded_bytes:config_encoded_bytes()
                metadata:comma_metadata_idx()? {
                    IrAstConfig {
                        value_name,
                        ty: val_ty,
                        encoded_bytes,
                        decode_fn,
                        metadata,
                    }
                }

            rule storage_key() -> IrAstStorageKey
                = "storage_key" _ namespaces:path() "." fields:field_access() _ "=" _ slot:constant() _ offset:storage_key_offset()? _ field_id:storage_key_field_id()? {
                    IrAstStorageKey {
                        namespaces,
                        fields,
                        slot,
                        offset,
                        field_id,
                    }
                }

            rule storage_key_offset() -> IrAstConst
                = ":" _ offset:constant() {
                    offset
                }

            rule storage_key_field_id() -> IrAstConst
                = ":" _ field_id:constant() {
                    field_id
                }

            rule fn_decl() -> IrAstFnDecl
                = is_public:is_public() _ is_original_entry:is_original_entry() _ is_entry:is_entry() _ is_fallback:is_fallback() _ "fn" _
                        name:id() _ selector:selector_id()? _ "(" _
                        args:(block_arg() ** comma()) ")" _ "->" _ ret_type:ast_ty()
                            metadata:comma_metadata_idx()? "{" _
                        locals:fn_local()*
                        blocks:block_decl()*
                    "}" _ {
                    // TODO: Remove once old decoding is removed.
                    //       In the case of old decoding, every entry is at the same time an original entry, but in the IR
                    //       we mark them only as `entry`s so there is a bit of information lost at the roundtrip.
                    //       Remove this hack to recognize the new encoding once it becomes the only encoding.
                    let is_original_entry = is_original_entry || (is_entry && !name.starts_with("__entry"));
                    IrAstFnDecl {
                        name,
                        args,
                        ret_type,
                        is_public,
                        metadata,
                        locals,
                        blocks,
                        selector,
                        is_entry,
                        is_original_entry,
                        is_fallback,
                    }
                }

            rule is_public() -> bool
                = "pub" _ { true }
                / "" _ { false }

            rule is_entry() -> bool
                = "entry" _ { true }
                / "" _ { false }

            rule is_original_entry() -> bool
                = "entry_orig" _ { true }
                / "" _ { false }

            rule is_fallback() -> bool
                = "fallback" _ { true }
                / "" _ { false }

            rule selector_id() -> [u8; 4]
                = "<" _ s:$(['0'..='9' | 'a'..='f' | 'A'..='F']*<8>) _ ">" _ {
                    string_to_hex::<4>(s)
                }

            rule block_arg() -> (IrAstTy, String, Option<MdIdxRef>)
                = name:id() mdi:metadata_idx()? ":" _ ty:ast_ty() {
                    (ty, name, mdi)
                }

            rule fn_local() -> (IrAstTy, String, Option<IrAstOperation>, bool)
                = "local" _ m:("mut" _)? ty:ast_ty() name:id() init:fn_local_init()? {
                    (ty, name, init, m.is_some())
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

            rule unary_op_kind() -> UnaryOpKind
                = "not" _ { UnaryOpKind::Not }

            rule binary_op_kind() -> BinaryOpKind
                = "add" _ { BinaryOpKind::Add }
                / "sub" _ { BinaryOpKind::Sub }
                / "mul" _ { BinaryOpKind::Mul }
                / "div" _ { BinaryOpKind::Div }
                / "and" _ { BinaryOpKind::And }
                / "or" _ { BinaryOpKind::Or }
                / "xor" _ { BinaryOpKind::Xor }
                / "mod" _ { BinaryOpKind::Mod }
                / "rsh" _ { BinaryOpKind::Rsh }
                / "lsh" _ { BinaryOpKind::Lsh }

            rule operation() -> IrAstOperation
                = op_asm()
                / op_wide_unary()
                / op_wide_binary()
                / op_wide_cmp()
                / op_retd()
                / op_branch()
                / op_bitcast()
                / op_unary()
                / op_binary()
                / op_call()
                / op_cast_ptr()
                / op_cbr()
                / op_switch()
                / op_cmp()
                / op_const()
                / op_contract_call()
                / op_get_elem_ptr()
                / op_get_local()
                / op_get_global()
                / op_get_config()
                / op_get_storage_key()
                / op_gtf()
                / op_int_to_ptr()
                / op_load()
                / op_log()
                / op_mem_copy_bytes()
                / op_mem_copy_val()
                / op_mem_clear_val()
                / op_nop()
                / op_ptr_to_int()
                / op_read_register()
                / op_ret()
                / op_revert()
                / op_jmp_mem()
                / op_smo()
                / op_state_load_quad_word()
                / op_state_load_word()
                / op_state_store_quad_word()
                / op_state_store_word()
                / op_store()
                / op_alloc()
                / op_init_aggr_array_repeat()
                / op_init_aggr()

            rule op_asm() -> IrAstOperation
                = "asm" _ "(" _ args:(asm_arg() ** comma()) ")" _ ret:asm_ret() meta_idx:comma_metadata_idx()? "{" _
                    ops:asm_op()*
                "}" _ {
                    IrAstOperation::Asm(
                        args,
                        ret.0,
                        ret.1,
                        ops,
                        meta_idx
                    )
                }

            rule op_bitcast() -> IrAstOperation
                = "bitcast" _ val:id() "to" _ ty:ast_ty() {
                    IrAstOperation::BitCast(val, ty)
                }

            rule op_unary() -> IrAstOperation
                = op: unary_op_kind() arg1:id() {
                    IrAstOperation::UnaryOp(op, arg1)
                }

            rule op_wide_modular_operation() -> IrAstOperation
                = "wide" _ op:binary_op_kind() arg1:id() comma() arg2:id() comma() arg3:id() "to" _ result:id()  {
                    IrAstOperation::WideModularOp(op, arg1, arg2, arg3, result)
                }

            rule op_wide_unary() -> IrAstOperation
                = "wide" _ op:unary_op_kind() arg:id() "to" _ result:id()  {
                    IrAstOperation::WideUnaryOp(op, arg, result)
                }

            rule op_wide_binary() -> IrAstOperation
                = "wide" _ op:binary_op_kind() arg1:id() comma() arg2:id() "to" _ result:id()  {
                    IrAstOperation::WideBinaryOp(op, arg1, arg2, result)
                }

            rule op_wide_cmp() -> IrAstOperation
                = "wide" _ "cmp" _ op:cmp_pred() arg1:id() arg2:id() {
                    IrAstOperation::WideCmp(op, arg1, arg2)
                }

            rule op_retd() -> IrAstOperation
                = "retd" _ arg1:id() _ arg2:id() {
                    IrAstOperation::Retd(arg1, arg2)
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

            rule op_cast_ptr() -> IrAstOperation
                = "cast_ptr" _ val:id() "to" _ ty:ast_ty() {
                    IrAstOperation::CastPtr(val, ty)
                }

            rule op_cbr() -> IrAstOperation
                = "cbr" _ cond:id() comma() tblock:id()
                "(" _ targs:(id() ** comma()) ")" _
                 comma() fblock:id() "(" _ fargs:(id() ** comma()) ")" _ {
                    IrAstOperation::Cbr(cond, tblock, targs, fblock, fargs)
                }

            rule switch_case() -> (u64, String, Vec<String>)
                = case_val:decimal() _ ":" _ case_block:id()
                "(" _ case_args:(id() ** comma()) _ ")" {
                    (case_val, case_block, case_args)
                }

            rule switch_default() -> Option<(String, Vec<String>)>
                = comma() "default" _ ":" _ dblock:id()
                "(" _ dargs:(id() ** comma()) ")" _ {
                    Some((dblock, dargs))
                }
                / { None }

            rule op_switch() -> IrAstOperation
                = "switch" _ discrim:id() _
                 default:switch_default()
                 comma() "[" _ cases:(switch_case() ** comma()) _ "]" _ {
                    IrAstOperation::Switch(discrim, default, cases)
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

            rule op_get_elem_ptr() -> IrAstOperation
                = "get_elem_ptr" _ base:id() comma() ty:ast_ty() comma() idcs:(id() ++ comma()) {
                    IrAstOperation::GetElemPtr(base, ty, idcs)
            }

            rule op_get_local() -> IrAstOperation
                = "get_local" _ ast_ty() comma() name:id() {
                    IrAstOperation::GetLocal(name)
                }

            rule op_get_global() -> IrAstOperation
                = "get_global" _ ast_ty() comma() name:path() {
                    IrAstOperation::GetGlobal(name)
                }

            rule op_get_config() -> IrAstOperation
                = "get_config" _ ast_ty() comma() name:id() {
                    IrAstOperation::GetConfig(name)
                }

            rule op_get_storage_key() -> IrAstOperation
                = "get_storage_key" _ ast_ty() comma() namespaces:path() "." fields:field_access() {
                    IrAstOperation::GetStorageKey(format!("{}.{}", namespaces.join("::"), fields.join(".")))
                }

            rule op_gtf() -> IrAstOperation
                = "gtf" _ index:id() comma() tx_field_id:decimal()  {
                    IrAstOperation::Gtf(index, tx_field_id)
                }

            rule op_int_to_ptr() -> IrAstOperation
                = "int_to_ptr" _ val:id() "to" _ ty:ast_ty() {
                    IrAstOperation::IntToPtr(val, ty)
                }

            rule op_alloc() -> IrAstOperation
                = "alloc" _ ty:ast_ty() "x" _ count:id() {
                    IrAstOperation::Alloc(ty, count)
                }

            rule op_load() -> IrAstOperation
                = "load" _ src:id() {
                    IrAstOperation::Load(src)
                }

            rule log_event_data() -> LogEventData
                = _ "log_data" _ "(" _
                    "version" _ ":" _ version:decimal() comma()
                    "is_event" _ ":" _ is_event:bool_lit() comma()
                    "is_indexed" _ ":" _ is_indexed:bool_lit() comma()
                    "event_type_size" _ ":" _ event_type_size:decimal() comma()
                    "num_elements" _ ":" _ num_elements:decimal()
                    _ ")" {
                    LogEventData::new(
                        u8::try_from(version).expect("log event data version must fit in u8"),
                        is_event,
                        is_indexed,
                        u8::try_from(event_type_size)
                            .expect("log event data size must fit in u8"),
                        u16::try_from(num_elements)
                            .expect("log event data count must fit in u16"),
                    )
                }

            rule op_log() -> IrAstOperation
                = "log" _ log_ty:ast_ty() log_val:id() comma() log_id:id() log_data:log_event_data()? _ {
                    IrAstOperation::Log(log_ty, log_val, log_id, log_data)
                }

            rule op_mem_copy_bytes() -> IrAstOperation
                = "mem_copy_bytes" _ dst_name:id() comma() src_name:id() comma() len:decimal() {
                    IrAstOperation::MemCopyBytes(dst_name, src_name, len)
                }

            rule op_mem_copy_val() -> IrAstOperation
                = "mem_copy_val" _ dst_name:id() comma() src_name:id() {
                    IrAstOperation::MemCopyVal(dst_name, src_name)
                }

            rule op_mem_clear_val() -> IrAstOperation
            = "mem_clear_val" _ dst_name:id() {
                IrAstOperation::MemClearVal(dst_name)
            }

            rule op_nop() -> IrAstOperation
                = "nop" _ {
                    IrAstOperation::Nop
                }

            rule op_ptr_to_int() -> IrAstOperation
                = "ptr_to_int" _ val:id() "to" _ ty:ast_ty() {
                    IrAstOperation::PtrToInt(val, ty)
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

            rule op_jmp_mem() -> IrAstOperation
                = "jmp_mem" _ {
                    IrAstOperation::JmpMem
                }

            rule op_smo() -> IrAstOperation
                = "smo" _
                recipient_and_message:id() comma() message_size:id() comma() output_index:id() comma() coins:id() _ {
                    IrAstOperation::Smo(recipient_and_message, message_size, output_index, coins)
            }

            rule op_state_clear() -> IrAstOperation
                = "state_clear" _ "key" _ key:id() comma()  number_of_slots:id() {
                    IrAstOperation::StateClear(key, number_of_slots)
                }

            rule op_state_load_quad_word() -> IrAstOperation
                = "state_load_quad_word" _ dst:id() comma() "key" _ key:id() comma()  number_of_slots:id() {
                    IrAstOperation::StateLoadQuadWord(dst, key, number_of_slots)
                }

            rule op_state_load_word() -> IrAstOperation
                = "state_load_word" _ "key" _ key:id() {
                    IrAstOperation::StateLoadWord(key)
                }

            rule op_state_store_quad_word() -> IrAstOperation
                = "state_store_quad_word" _ src:id() comma() "key" _ key:id() comma()  number_of_slots:id() {
                    IrAstOperation::StateStoreQuadWord(src, key, number_of_slots)
                }

            rule op_state_store_word() -> IrAstOperation
                = "state_store_word" _ src:id() comma() "key" _ key:id() {
                    IrAstOperation::StateStoreWord(src, key)
                }

            rule op_store() -> IrAstOperation
                = "store" _ val:id() "to" _ dst:id() {
                    IrAstOperation::Store(val, dst)
                }

            rule op_init_aggr_array_repeat() -> IrAstOperation
                = "init_aggr" _ aggr_ptr:id() "[" _ repeated_value:id() _ "x" _ length:decimal() _ "]" _ {
                    IrAstOperation::InitAggr(aggr_ptr, std::iter::repeat_n(repeated_value, length as usize).collect())
                }

            rule op_init_aggr() -> IrAstOperation
                = "init_aggr" _ aggr_ptr:id() "[" _ initializers:(id() ** comma()) "]" _ {
                    IrAstOperation::InitAggr(aggr_ptr, initializers)
                }

            rule cmp_pred() -> Predicate
                = "eq" _ { Predicate::Equal }
                / "gt" _ { Predicate::GreaterThan }
                / "lt" _ { Predicate::LessThan }

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

            rule asm_ret() -> (IrAstTy, Option<Ident>)
                = "->" _ ty:ast_ty() ret:id_id()? {
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
                    IrAstConstValue::Hex256(string_to_hex::<32>(s))
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
                    (ty.clone(), IrAstConst { value: IrAstConstValue::Undef, meta_idx: None })
                }

            rule ast_ty() -> IrAstTy
                = ("unit" / "()") _ { IrAstTy::Unit }
                / "bool" _ { IrAstTy::Bool }
                / "u8" _ { IrAstTy::U8 }
                / "u64" _ { IrAstTy::U64 }
                / "u256" _ { IrAstTy::U256 }
                / "b256" _ { IrAstTy::B256 }
                / "slice" _ { IrAstTy::Slice }
                / "__slice" _ "[" _ ty:ast_ty() "]" _ { IrAstTy::TypedSlice(Box::new(ty)) }
                / "string" _ "<" _ sz:decimal() ">" _ { IrAstTy::String(sz) }
                / array_ty()
                / struct_ty()
                / union_ty()
                / "__ptr" _ ty:ast_ty() _ { IrAstTy::TypedPtr(Box::new(ty)) }
                / "ptr" _ { IrAstTy::Ptr }
                / "never" _ { IrAstTy::Never }

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
                = !(ast_ty() (" " "\n")) id:$(id_char0() id_char()*) _ {
                    id.to_owned()
                }

            rule id_id() -> Ident
                = !(ast_ty() (" " "\n")) id:$(id_char0() id_char()*) _ {
                    Ident::new(Span::new(id.into(), 0, id.len(), None).unwrap())
                }

            rule path() -> Vec<String>
                = (id() ** "::")

            rule field_access() -> Vec<String>
                = (id() ** ".")

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
                    // Metadata strings are printed with '\\' escaped on parsing we unescape it.
                    IrMetadatum::String(s.to_owned().replace("\\\\", "\\"))
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

            rule bool_lit() -> bool
                = "true" _ { true }
                / "false" _ { false }

            // String of decimal digits without discarding whitespace. (Useful for newline
            // sensitive metadata).
            rule dec_digits() -> u64
                = ds:$("0" / ['1'..='9'] ['0'..='9']*) {
                    ds.parse::<u64>().unwrap()
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
        constant::{ConstantContent, ConstantValue},
        context::Context,
        error::IrError,
        function::Function,
        instruction::{InstOp, Predicate, Register},
        irtype::Type,
        metadata::{MetadataIndex, Metadatum},
        module::{Kind, Module},
        value::Value,
        variable::LocalVar,
        Backtrace, BinaryOpKind, BlockArgument, BranchToWithArgs, ConfigContent, Constant,
        GlobalVar, Instruction, LogEventData, StorageKey, UnaryOpKind, B256,
    };

    #[derive(Debug)]
    pub(super) struct IrAstModule {
        kind: Kind,
        configs: Vec<IrAstConfig>,
        global_vars: Vec<IrAstGlobalVar>,
        storage_keys: Vec<IrAstStorageKey>,
        fn_decls: Vec<IrAstFnDecl>,
        metadata: Vec<(MdIdxRef, IrMetadatum)>,
    }

    #[derive(Debug)]
    pub(super) struct IrAstGlobalVar {
        name: Vec<String>,
        ty: IrAstTy,
        init: Option<IrAstOperation>,
        mutable: bool,
    }

    #[derive(Debug)]
    pub(super) struct IrAstStorageKey {
        namespaces: Vec<String>,
        fields: Vec<String>,
        slot: IrAstConst,
        offset: Option<IrAstConst>,
        field_id: Option<IrAstConst>,
    }

    #[derive(Debug)]
    struct IrAstFnDecl {
        name: String,
        args: Vec<(IrAstTy, String, Option<MdIdxRef>)>,
        ret_type: IrAstTy,
        is_public: bool,
        metadata: Option<MdIdxRef>,
        locals: Vec<(IrAstTy, String, Option<IrAstOperation>, bool)>,
        blocks: Vec<IrAstBlock>,
        selector: Option<[u8; 4]>,
        is_entry: bool,
        is_original_entry: bool,
        is_fallback: bool,
    }

    #[derive(Debug)]
    struct IrAstBlock {
        label: String,
        args: Vec<(IrAstTy, String, Option<MdIdxRef>)>,
        instructions: Vec<IrAstInstruction>,
    }

    #[derive(Debug)]
    struct IrAstInstruction {
        value_name: Option<String>,
        op: IrAstOperation,
        metadata: Option<MdIdxRef>,
    }

    #[derive(Debug)]
    enum IrAstOperation {
        Asm(
            Vec<(Ident, Option<IrAstAsmArgInit>)>,
            IrAstTy,
            Option<Ident>,
            Vec<IrAstAsmOp>,
            Option<MdIdxRef>,
        ),
        BitCast(String, IrAstTy),
        UnaryOp(UnaryOpKind, String),
        BinaryOp(BinaryOpKind, String, String),
        Br(String, Vec<String>),
        Call(String, Vec<String>),
        CastPtr(String, IrAstTy),
        Cbr(String, String, Vec<String>, String, Vec<String>),
        // (descriminant, default_block, default_args, [(u64, block, args)])
        Switch(
            String,
            Option<(String, Vec<String>)>,
            Vec<(u64, String, Vec<String>)>,
        ),
        Cmp(Predicate, String, String),
        Const(IrAstTy, IrAstConst),
        ContractCall(IrAstTy, String, String, String, String, String),
        GetElemPtr(String, IrAstTy, Vec<String>),
        GetLocal(String),
        GetGlobal(Vec<String>),
        GetConfig(String),
        GetStorageKey(String),
        Gtf(String, u64),
        IntToPtr(String, IrAstTy),
        Load(String),
        Log(IrAstTy, String, String, Option<LogEventData>),
        MemCopyBytes(String, String, u64),
        MemCopyVal(String, String),
        MemClearVal(String),
        Nop,
        PtrToInt(String, IrAstTy),
        ReadRegister(String),
        Ret(IrAstTy, String),
        Revert(String),
        JmpMem,
        Smo(String, String, String, String),
        StateClear(String, String),
        StateLoadQuadWord(String, String, String),
        StateLoadWord(String),
        StateStoreQuadWord(String, String, String),
        StateStoreWord(String, String),
        Store(String, String),
        WideUnaryOp(UnaryOpKind, String, String),
        WideBinaryOp(BinaryOpKind, String, String, String),
        WideCmp(Predicate, String, String),
        WideModularOp(BinaryOpKind, String, String, String, String),
        Retd(String, String),
        Alloc(IrAstTy, String),
        InitAggr(String, Vec<String>),
    }

    #[derive(Debug)]
    struct IrAstConfig {
        value_name: String,
        ty: IrAstTy,
        encoded_bytes: Vec<u8>,
        decode_fn: String,
        metadata: Option<MdIdxRef>,
    }

    #[derive(Debug)]
    struct IrAstConst {
        value: IrAstConstValue,
        meta_idx: Option<MdIdxRef>,
    }

    #[derive(Debug)]
    enum IrAstConstValue {
        Undef,
        Unit,
        Bool(bool),
        Hex256([u8; 32]),
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
        name: Ident,
        args: Vec<Ident>,
        imm: Option<Ident>,
        meta_idx: Option<MdIdxRef>,
    }

    impl IrAstConstValue {
        fn as_constant_value(&self, context: &mut Context, val_ty: IrAstTy) -> ConstantValue {
            match self {
                IrAstConstValue::Undef => ConstantValue::Undef,
                IrAstConstValue::Unit => ConstantValue::Unit,
                IrAstConstValue::Bool(b) => ConstantValue::Bool(*b),
                IrAstConstValue::Hex256(bs) => match val_ty {
                    IrAstTy::U256 => {
                        let value = U256::from_be_bytes(bs);
                        ConstantValue::U256(value)
                    }
                    IrAstTy::B256 => {
                        let value = B256::from_be_bytes(bs);
                        ConstantValue::B256(value)
                    }
                    _ => unreachable!("invalid type for hex number"),
                },
                IrAstConstValue::Number(n) => ConstantValue::Uint(*n),
                IrAstConstValue::String(bs) => ConstantValue::String(bs.clone()),
                IrAstConstValue::Array(el_ty, els) => {
                    let els: Vec<_> = els
                        .iter()
                        .map(|cv| {
                            cv.value
                                .as_constant(context, el_ty.clone())
                                .get_content(context)
                                .clone()
                        })
                        .collect();
                    ConstantValue::Array(els)
                }
                IrAstConstValue::Struct(flds) => {
                    let fields: Vec<_> = flds
                        .iter()
                        .map(|(ty, cv)| {
                            cv.value
                                .as_constant(context, ty.clone())
                                .get_content(context)
                                .clone()
                        })
                        .collect::<Vec<_>>();
                    ConstantValue::Struct(fields)
                }
            }
        }

        fn as_constant(&self, context: &mut Context, val_ty: IrAstTy) -> Constant {
            let value = self.as_constant_value(context, val_ty.clone());
            let constant = ConstantContent {
                ty: val_ty.to_ir_type(context),
                value,
            };
            Constant::unique(context, constant)
        }

        fn as_value(&self, context: &mut Context, val_ty: IrAstTy) -> Value {
            match self {
                IrAstConstValue::Undef => unreachable!("Can't convert 'undef' to a value."),
                IrAstConstValue::Unit => ConstantContent::get_unit(context),
                IrAstConstValue::Bool(b) => ConstantContent::get_bool(context, *b),
                IrAstConstValue::Hex256(bs) => match val_ty {
                    IrAstTy::U256 => {
                        let n = U256::from_be_bytes(bs);
                        ConstantContent::get_uint256(context, n)
                    }
                    IrAstTy::B256 => ConstantContent::get_b256(context, *bs),
                    _ => unreachable!("invalid type for hex number"),
                },
                IrAstConstValue::Number(n) => match val_ty {
                    IrAstTy::U8 => ConstantContent::get_uint(context, 8, *n),
                    IrAstTy::U64 => ConstantContent::get_uint(context, 64, *n),
                    _ => unreachable!(),
                },
                IrAstConstValue::String(s) => ConstantContent::get_string(context, s.clone()),
                IrAstConstValue::Array(..) => {
                    let array_const = self.as_constant(context, val_ty);
                    ConstantContent::get_array(context, array_const.get_content(context).clone())
                }
                IrAstConstValue::Struct(_) => {
                    let struct_const = self.as_constant(context, val_ty);
                    ConstantContent::get_struct(context, struct_const.get_content(context).clone())
                }
            }
        }
    }

    #[derive(Clone, Debug)]
    enum IrAstTy {
        Unit,
        Bool,
        U8,
        U64,
        U256,
        B256,
        Slice,
        TypedSlice(Box<IrAstTy>),
        String(u64),
        Array(Box<IrAstTy>, u64),
        Union(Vec<IrAstTy>),
        Struct(Vec<IrAstTy>),
        TypedPtr(Box<IrAstTy>),
        Ptr,
        Never,
    }

    impl IrAstTy {
        fn to_ir_type(&self, context: &mut Context) -> Type {
            match self {
                IrAstTy::Unit => Type::get_unit(context),
                IrAstTy::Bool => Type::get_bool(context),
                IrAstTy::U8 => Type::get_uint8(context),
                IrAstTy::U64 => Type::get_uint64(context),
                IrAstTy::U256 => Type::get_uint256(context),
                IrAstTy::B256 => Type::get_b256(context),
                IrAstTy::Slice => Type::get_slice(context),
                IrAstTy::TypedSlice(el_ty) => {
                    let inner_ty = el_ty.to_ir_type(context);
                    Type::get_typed_slice(context, inner_ty)
                }
                IrAstTy::String(n) => Type::new_string_array(context, *n),
                IrAstTy::Array(el_ty, count) => {
                    let el_ty = el_ty.to_ir_type(context);
                    Type::new_array(context, el_ty, *count)
                }
                IrAstTy::Union(tys) => {
                    let tys = tys.iter().map(|ty| ty.to_ir_type(context)).collect();
                    Type::new_union(context, tys)
                }
                IrAstTy::Struct(tys) => {
                    let tys = tys.iter().map(|ty| ty.to_ir_type(context)).collect();
                    Type::new_struct(context, tys)
                }
                IrAstTy::TypedPtr(ty) => {
                    let inner_ty = ty.to_ir_type(context);
                    Type::new_typed_pointer(context, inner_ty)
                }
                IrAstTy::Ptr => Type::get_ptr(context),
                IrAstTy::Never => Type::get_never(context),
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

    use std::{
        cell::Cell,
        collections::{BTreeMap, HashMap},
        iter::FromIterator,
    };

    pub(super) fn build_context(
        ir_ast_mod: IrAstModule,
        source_engine: &SourceEngine,
        experimental: ExperimentalFeatures,
        backtrace: Backtrace,
    ) -> Result<Context, IrError> {
        let mut ctx = Context::new(source_engine, experimental, backtrace);
        let md_map = build_metadata_map(&mut ctx, ir_ast_mod.metadata);
        let module = Module::new(&mut ctx, ir_ast_mod.kind);
        let mut builder = IrBuilder {
            module,
            configs_map: build_configs_map(&mut ctx, &module, ir_ast_mod.configs, &md_map),
            globals_map: build_global_vars_map(&mut ctx, &module, ir_ast_mod.global_vars),
            storage_keys_map: build_storage_keys_map(&mut ctx, &module, ir_ast_mod.storage_keys),
            md_map,
            unresolved_calls: Vec::new(),
        };

        for fn_decl in ir_ast_mod.fn_decls {
            builder.add_fn_decl(&mut ctx, fn_decl)?;
        }

        builder.resolve_calls(&mut ctx)?;

        Ok(ctx)
    }

    struct IrBuilder {
        module: Module,
        configs_map: BTreeMap<String, String>,
        globals_map: BTreeMap<Vec<String>, GlobalVar>,
        storage_keys_map: BTreeMap<String, StorageKey>,
        md_map: HashMap<MdIdxRef, MetadataIndex>,
        unresolved_calls: Vec<PendingCall>,
    }

    struct PendingCall {
        call_val: Value,
        callee: String,
    }

    impl IrBuilder {
        fn add_fn_decl(
            &mut self,
            context: &mut Context,
            fn_decl: IrAstFnDecl,
        ) -> Result<(), IrError> {
            let convert_md_idx = |opt_md_idx: &Option<MdIdxRef>| {
                opt_md_idx.and_then(|mdi| self.md_map.get(&mdi).copied())
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
                fn_decl.name.clone(),
                fn_decl.name,
                args,
                ret_type,
                fn_decl.selector,
                fn_decl.is_public,
                fn_decl.is_entry,
                fn_decl.is_original_entry,
                fn_decl.is_fallback,
                convert_md_idx(&fn_decl.metadata),
            );

            let mut arg_map = HashMap::default();
            let mut local_map = HashMap::<String, LocalVar>::new();
            for (ty, name, initializer, mutable) in fn_decl.locals {
                let initializer = initializer.map(|const_init| {
                    if let IrAstOperation::Const(val_ty, val) = const_init {
                        val.value.as_constant(context, val_ty)
                    } else {
                        unreachable!("BUG! Initializer must be a const value.");
                    }
                });
                let ty = ty.to_ir_type(context);
                local_map.insert(
                    name.clone(),
                    func.new_local_var(context, name, ty, initializer, mutable)?,
                );
            }

            // The entry block is already created, we don't want to recreate it.
            let named_blocks =
                HashMap::from_iter(fn_decl.blocks.iter().scan(true, |is_entry, block| {
                    Some((
                        block.label.clone(),
                        if *is_entry {
                            *is_entry = false;
                            func.get_entry_block(context)
                        } else {
                            let irblock = func.create_block(context, Some(block.label.clone()));
                            for (idx, (arg_ty, _, md)) in block.args.iter().enumerate() {
                                let ty = arg_ty.to_ir_type(context);
                                let arg = Value::new_argument(
                                    context,
                                    BlockArgument {
                                        block: irblock,
                                        idx,
                                        ty,
                                        // TODO: Support immutable flag on block arguments.
                                        is_immutable: false,
                                    },
                                )
                                .add_metadatum(context, convert_md_idx(md));
                                irblock.add_arg(context, arg);
                            }
                            irblock
                        },
                    ))
                }));

            for block in fn_decl.blocks {
                for (idx, arg) in block.args.iter().enumerate() {
                    arg_map.insert(
                        arg.1.clone(),
                        named_blocks[&block.label].get_arg(context, idx).unwrap(),
                    );
                }
                self.add_block_instructions(
                    context,
                    block,
                    &named_blocks,
                    &local_map,
                    &mut arg_map,
                );
            }
            Ok(())
        }

        fn add_block_instructions(
            &mut self,
            context: &mut Context,
            ir_block: IrAstBlock,
            named_blocks: &HashMap<String, Block>,
            local_map: &HashMap<String, LocalVar>,
            val_map: &mut HashMap<String, Value>,
        ) {
            let block = named_blocks.get(&ir_block.label).unwrap();
            for ins in ir_block.instructions {
                let opt_metadata = ins.metadata.and_then(|mdi| self.md_map.get(&mdi)).copied();
                let ins_val = match ins.op {
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
                                    op_name: name,
                                    args,
                                    immediate: imm,
                                    metadata: meta_idx
                                        .as_ref()
                                        .and_then(|meta_idx| self.md_map.get(meta_idx).copied()),
                                },
                            )
                            .collect();
                        let md_idx = meta_idx.and_then(|mdi| self.md_map.get(&mdi)).copied();
                        let return_type = return_type.to_ir_type(context);
                        block
                            .append(context)
                            .asm_block(args, body, return_type, return_name)
                            .add_metadatum(context, md_idx)
                    }
                    IrAstOperation::BitCast(val, ty) => {
                        let to_ty = ty.to_ir_type(context);
                        block
                            .append(context)
                            .bitcast(*val_map.get(&val).unwrap(), to_ty)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::Alloc(ty, name) => {
                        let ir_ty = ty.to_ir_type(context);
                        block
                            .append(context)
                            .alloc(ir_ty, *val_map.get(&name).unwrap())
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::UnaryOp(op, arg) => block
                        .append(context)
                        .unary_op(op, *val_map.get(&arg).unwrap())
                        .add_metadatum(context, opt_metadata),
                    // Wide Operations
                    IrAstOperation::WideUnaryOp(op, arg, result) => block
                        .append(context)
                        .wide_unary_op(
                            op,
                            *val_map.get(&arg).unwrap(),
                            *val_map.get(&result).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::WideBinaryOp(op, arg1, arg2, result) => block
                        .append(context)
                        .wide_binary_op(
                            op,
                            *val_map.get(&arg1).unwrap(),
                            *val_map.get(&arg2).unwrap(),
                            *val_map.get(&result).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::WideModularOp(op, arg1, arg2, arg3, result) => block
                        .append(context)
                        .wide_modular_op(
                            op,
                            *val_map.get(&result).unwrap(),
                            *val_map.get(&arg1).unwrap(),
                            *val_map.get(&arg2).unwrap(),
                            *val_map.get(&arg3).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::WideCmp(op, arg1, arg2) => block
                        .append(context)
                        .wide_cmp_op(
                            op,
                            *val_map.get(&arg1).unwrap(),
                            *val_map.get(&arg2).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Retd(ret_ptr, ret_len) => block
                        .append(context)
                        .retd(
                            *val_map.get(&ret_ptr).unwrap(),
                            *val_map.get(&ret_len).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::BinaryOp(op, arg1, arg2) => block
                        .append(context)
                        .binary_op(
                            op,
                            *val_map.get(&arg1).unwrap(),
                            *val_map.get(&arg2).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Br(to_block_name, args) => {
                        let to_block = named_blocks.get(&to_block_name).unwrap();
                        block
                            .append(context)
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
                            .append(context)
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
                    IrAstOperation::CastPtr(val, ty) => {
                        let ir_ty = ty.to_ir_type(context);
                        block
                            .append(context)
                            .cast_ptr(*val_map.get(&val).unwrap(), ir_ty)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::Cbr(
                        cond_val_name,
                        true_block_name,
                        true_args,
                        false_block_name,
                        false_args,
                    ) => block
                        .append(context)
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
                    IrAstOperation::Switch(discrim_name, default, cases) => {
                        let descrim_val = *val_map.get(&discrim_name).unwrap();
                        let default = default.map(|(block_name, block_args)| {
                            let block = *named_blocks.get(&block_name).unwrap();
                            let args = block_args
                                .iter()
                                .map(|arg| *val_map.get(arg).unwrap())
                                .collect();
                            BranchToWithArgs { block, args }
                        });
                        let case_blocks: Vec<(u64, BranchToWithArgs)> = cases
                            .into_iter()
                            .map(|(case_val, block_name, block_args)| {
                                let block = *named_blocks.get(&block_name).unwrap();
                                let args = block_args
                                    .into_iter()
                                    .map(|arg| *val_map.get(&arg).unwrap())
                                    .collect();
                                (case_val, BranchToWithArgs { block, args })
                            })
                            .collect();
                        block
                            .append(context)
                            .switch(descrim_val, default, case_blocks)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::Cmp(pred, lhs, rhs) => block
                        .append(context)
                        .cmp(
                            pred,
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
                            .append(context)
                            .contract_call(
                                ir_ty,
                                Some(name),
                                *val_map.get(&params).unwrap(),
                                *val_map.get(&coins).unwrap(),
                                *val_map.get(&asset_id).unwrap(),
                                *val_map.get(&gas).unwrap(),
                            )
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::GetElemPtr(base, elem_ty, idcs) => {
                        let ir_elem_ty = elem_ty
                            .to_ir_type(context)
                            .get_pointee_type(context)
                            .unwrap();
                        block
                            .append(context)
                            .get_elem_ptr(
                                *val_map.get(&base).unwrap(),
                                ir_elem_ty,
                                idcs.iter().map(|idx| *val_map.get(idx).unwrap()).collect(),
                            )
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::GetLocal(local_name) => block
                        .append(context)
                        .get_local(*local_map.get(&local_name).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::GetGlobal(global_name) => block
                        .append(context)
                        .get_global(*self.globals_map.get(&global_name).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::GetConfig(name) => block
                        .append(context)
                        .get_config(self.module, name)
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::GetStorageKey(path) => block
                        .append(context)
                        .get_storage_key(*self.storage_keys_map.get(&path).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Gtf(index, tx_field_id) => block
                        .append(context)
                        .gtf(*val_map.get(&index).unwrap(), tx_field_id)
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::IntToPtr(val, ty) => {
                        let to_ty = ty.to_ir_type(context);
                        block
                            .append(context)
                            .int_to_ptr(*val_map.get(&val).unwrap(), to_ty)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::Load(src_name) => block
                        .append(context)
                        .load(*val_map.get(&src_name).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Log(log_ty, log_val, log_id, log_data) => {
                        let log_ty = log_ty.to_ir_type(context);
                        block
                            .append(context)
                            .log(
                                *val_map.get(&log_val).unwrap(),
                                log_ty,
                                *val_map.get(&log_id).unwrap(),
                                log_data,
                            )
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::MemCopyBytes(dst_name, src_name, len) => block
                        .append(context)
                        .mem_copy_bytes(
                            *val_map.get(&dst_name).unwrap(),
                            *val_map.get(&src_name).unwrap(),
                            len,
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::MemCopyVal(dst_name, src_name) => block
                        .append(context)
                        .mem_copy_val(
                            *val_map.get(&dst_name).unwrap(),
                            *val_map.get(&src_name).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::MemClearVal(dst_name) => block
                        .append(context)
                        .mem_clear_val(*val_map.get(&dst_name).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Nop => block.append(context).nop(),
                    IrAstOperation::PtrToInt(val, ty) => {
                        let to_ty = ty.to_ir_type(context);
                        block
                            .append(context)
                            .ptr_to_int(*val_map.get(&val).unwrap(), to_ty)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::ReadRegister(reg_name) => block
                        .append(context)
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
                            .append(context)
                            .ret(*val_map.get(&ret_val_name).unwrap(), ty)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::Revert(ret_val_name) => block
                        .append(context)
                        .revert(*val_map.get(&ret_val_name).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::JmpMem => block
                        .append(context)
                        .jmp_mem()
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Smo(recipient, message, message_size, coins) => block
                        .append(context)
                        .smo(
                            *val_map.get(&recipient).unwrap(),
                            *val_map.get(&message).unwrap(),
                            *val_map.get(&message_size).unwrap(),
                            *val_map.get(&coins).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::StateClear(key, number_of_slots) => block
                        .append(context)
                        .state_clear(
                            *val_map.get(&key).unwrap(),
                            *val_map.get(&number_of_slots).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::StateLoadQuadWord(dst, key, number_of_slots) => block
                        .append(context)
                        .state_load_quad_word(
                            *val_map.get(&dst).unwrap(),
                            *val_map.get(&key).unwrap(),
                            *val_map.get(&number_of_slots).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::StateLoadWord(key) => block
                        .append(context)
                        .state_load_word(*val_map.get(&key).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::StateStoreQuadWord(src, key, number_of_slots) => block
                        .append(context)
                        .state_store_quad_word(
                            *val_map.get(&src).unwrap(),
                            *val_map.get(&key).unwrap(),
                            *val_map.get(&number_of_slots).unwrap(),
                        )
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::StateStoreWord(src, key) => block
                        .append(context)
                        .state_store_word(*val_map.get(&src).unwrap(), *val_map.get(&key).unwrap())
                        .add_metadatum(context, opt_metadata),
                    IrAstOperation::Store(stored_val_name, dst_val_name) => {
                        let dst_val_ptr = *val_map.get(&dst_val_name).unwrap();
                        let stored_val = *val_map.get(&stored_val_name).unwrap();

                        block
                            .append(context)
                            .store(dst_val_ptr, stored_val)
                            .add_metadatum(context, opt_metadata)
                    }
                    IrAstOperation::InitAggr(aggr_ptr, initializers) => {
                        let aggr_ptr_val = *val_map.get(&aggr_ptr).unwrap();
                        let init_vals: Vec<Value> = initializers
                            .iter()
                            .map(|init_name| *val_map.get(init_name).unwrap())
                            .collect();
                        block
                            .append(context)
                            .init_aggr(aggr_ptr_val, init_vals)
                            .add_metadatum(context, opt_metadata)
                    }
                };
                ins.value_name.map(|vn| val_map.insert(vn, ins_val));
            }
        }

        fn resolve_calls(self, context: &mut Context) -> Result<(), IrError> {
            for (configurable_name, fn_name) in self.configs_map {
                let f = self
                    .module
                    .function_iter(context)
                    .find(|x| x.get_name(context) == fn_name)
                    .unwrap();

                if let Some(ConfigContent::V1 { decode_fn, .. }) = context
                    .modules
                    .get_mut(self.module.0)
                    .unwrap()
                    .configs
                    .get_mut(&configurable_name)
                {
                    decode_fn.replace(f);
                }
            }

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

                if let Some(Instruction {
                    op: InstOp::Call(dummy_func, _args),
                    ..
                }) = pending_call.call_val.get_instruction_mut(context)
                {
                    *dummy_func = call_func;
                }
            }
            Ok(())
        }
    }

    fn build_global_vars_map(
        context: &mut Context,
        module: &Module,
        global_vars: Vec<IrAstGlobalVar>,
    ) -> BTreeMap<Vec<String>, GlobalVar> {
        global_vars
            .into_iter()
            .map(|global_var_node| {
                let ty = global_var_node.ty.to_ir_type(context);
                let init = global_var_node.init.map(|init| match init {
                    IrAstOperation::Const(ty, val) => val.value.as_constant(context, ty),
                    _ => unreachable!("Global const initializer must be a const value."),
                });
                let global_var = GlobalVar::new(context, ty, init, global_var_node.mutable);
                module.add_global_variable(context, global_var_node.name.clone(), global_var);
                (global_var_node.name, global_var)
            })
            .collect()
    }

    fn build_storage_keys_map(
        context: &mut Context,
        module: &Module,
        storage_keys: Vec<IrAstStorageKey>,
    ) -> BTreeMap<String, StorageKey> {
        storage_keys
            .into_iter()
            .map(|storage_key_node| {
                let path = format!(
                    "{}.{}",
                    storage_key_node.namespaces.join("::"),
                    storage_key_node.fields.join("."),
                );
                let slot = match storage_key_node.slot.value {
                    IrAstConstValue::Hex256(val) => val,
                    _ => panic!("Storage key slot must be a hex string representing b256."),
                };
                let offset = match storage_key_node.offset {
                    Some(IrAstConst {
                        value: IrAstConstValue::Number(n),
                        ..
                    }) => n,
                    None => 0,
                    _ => panic!("Storage key offset must be a u64 constant."),
                };
                let field_id = match storage_key_node.field_id {
                    Some(IrAstConst {
                        value: IrAstConstValue::Hex256(val),
                        ..
                    }) => val,
                    // If the field id is not specified, it is the same as the slot.
                    None => slot,
                    _ => panic!("Storage key field id must be a hex string representing b256."),
                };
                let storage_key = StorageKey::new(context, slot, offset, field_id);
                module.add_storage_key(context, path.clone(), storage_key);
                (path, storage_key)
            })
            .collect()
    }

    fn build_configs_map(
        context: &mut Context,
        module: &Module,
        configs: Vec<IrAstConfig>,
        md_map: &HashMap<MdIdxRef, MetadataIndex>,
    ) -> BTreeMap<String, String> {
        configs
            .into_iter()
            .map(|config| {
                let opt_metadata = config
                    .metadata
                    .map(|mdi| md_map.get(&mdi).unwrap())
                    .copied();

                let ty = config.ty.to_ir_type(context);

                let config_val = ConfigContent::V1 {
                    name: config.value_name.clone(),
                    ty,
                    ptr_ty: Type::new_typed_pointer(context, ty),
                    encoded_bytes: config.encoded_bytes,
                    // this will point to the correct function after all functions are compiled
                    decode_fn: Cell::new(Function(KeyData::default().into())),
                    opt_metadata,
                };

                module.add_config(context, config.value_name.clone(), config_val.clone());

                (config.value_name.clone(), config.decode_fn.clone())
            })
            .collect()
    }

    /// Create the metadata for the module in `context` and generate a map from the parsed
    /// `MdIdxRef`s to the new actual metadata.
    fn build_metadata_map(
        context: &mut Context,
        ir_metadata: Vec<(MdIdxRef, IrMetadatum)>,
    ) -> HashMap<MdIdxRef, MetadataIndex> {
        fn convert_md(md: IrMetadatum, md_map: &mut HashMap<MdIdxRef, MetadataIndex>) -> Metadatum {
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
                        .map(|el_md| convert_md(el_md, md_map))
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
            let md = convert_md(ir_md, &mut md_map);
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

    fn hex_string_to_vec(s: &str) -> Vec<u8> {
        let mut bytes = vec![];
        let mut cur_byte: u8 = 0;
        for (idx, ch) in s.chars().enumerate() {
            cur_byte = (cur_byte << 4) | ch.to_digit(16).unwrap() as u8;
            if idx % 2 == 1 {
                bytes.push(cur_byte);
                cur_byte = 0;
            }
        }
        bytes
    }
}

// -------------------------------------------------------------------------------------------------
