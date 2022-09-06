use crate::{ParseErrorKind, ParseResult, Parser};

use sway_ast::expr::op_code::*;
use sway_types::{Ident, Spanned};

macro_rules! define_op_codes (
    ( $(($op_name:ident, $ty_name:ident, $s:literal, ($($arg_name:ident),*)),)* ) => {
        pub fn parse_instruction(ident: Ident, parser: &mut Parser) -> ParseResult<Instruction> {
            match ident.as_str() {
                $($s => {
                    $(
                        let $arg_name = parser.parse()?;
                    )*
                    Ok($ty_name::instruction(ident, ($($arg_name,)*)))
                },)*
                _ => {
                    let span = ident.span().clone();
                    Err(parser.emit_error_with_span(ParseErrorKind::UnrecognizedOpCode, span))
                },
            }
        }
    };
);

define_op_codes!(
    (Add, AddOpcode, "add", (ret, lhs, rhs)),
    (Addi, AddiOpcode, "addi", (ret, lhs, rhs)),
    (And, AndOpcode, "and", (ret, lhs, rhs)),
    (Andi, AndiOpcode, "andi", (ret, lhs, rhs)),
    (Div, DivOpcode, "div", (ret, lhs, rhs)),
    (Divi, DiviOpcode, "divi", (ret, lhs, rhs)),
    (Eq, EqOpcode, "eq", (ret, lhs, rhs)),
    (Exp, ExpOpcode, "exp", (ret, base, power)),
    (Expi, ExpiOpcode, "expi", (ret, base, power)),
    (Gt, GtOpcode, "gt", (ret, lhs, rhs)),
    (Gtf, GtfOpcode, "gtf", (ret, index, tx_field_id)),
    (Lt, LtOpcode, "lt", (ret, lhs, rhs)),
    (Mlog, MlogOpcode, "mlog", (ret, arg, base)),
    (Mod, ModOpcode, "mod", (ret, lhs, rhs)),
    (Modi, ModiOpcode, "modi", (ret, lhs, rhs)),
    (Move, MoveOpcode, "move", (ret, from)),
    (Movi, MoviOpcode, "movi", (ret, arg)),
    (Mroo, MrooOpcode, "mroo", (ret, arg, root)),
    (Mul, MulOpcode, "mul", (ret, lhs, rhs)),
    (Muli, MuliOpcode, "muli", (ret, lhs, rhs)),
    (Noop, NoopOpcode, "noop", ()),
    (Not, NotOpcode, "not", (ret, arg)),
    (Or, OrOpcode, "or", (ret, lhs, rhs)),
    (Ori, OriOpcode, "ori", (ret, lhs, rhs)),
    (Sll, SllOpcode, "sll", (ret, lhs, rhs)),
    (Slli, SlliOpcode, "slli", (ret, lhs, rhs)),
    (Smo, SmoOpcode, "smo", (addr, len, output, coins)),
    (Srl, SrlOpcode, "srl", (ret, lhs, rhs)),
    (Srli, SrliOpcode, "srli", (ret, lhs, rhs)),
    (Sub, SubOpcode, "sub", (ret, lhs, rhs)),
    (Subi, SubiOpcode, "subi", (ret, lhs, rhs)),
    (Xor, XorOpcode, "xor", (ret, lhs, rhs)),
    (Xori, XoriOpcode, "xori", (ret, lhs, rhs)),
    (Ret, RetOpcode, "ret", (value)),
    (Aloc, AlocOpcode, "aloc", (size)),
    (Cfei, CfeiOpcode, "cfei", (size)),
    (Cfsi, CfsiOpcode, "cfsi", (size)),
    (Lb, LbOpcode, "lb", (ret, addr, offset)),
    (Lw, LwOpcode, "lw", (ret, addr, offset)),
    (Mcl, MclOpcode, "mcl", (addr, size)),
    (Mcli, McliOpcode, "mcli", (addr, size)),
    (Mcp, McpOpcode, "mcp", (dst_addr, src_addr, size)),
    (Mcpi, McpiOpcode, "mcpi", (dst_addr, src_addr, size)),
    (Meq, MeqOpcode, "meq", (ret, lhs_addr, rhs_addr, size)),
    (Sb, SbOpcode, "sb", (addr, value, offset)),
    (Sw, SwOpcode, "sw", (addr, value, offset)),
    (Bal, BalOpcode, "bal", (ret, asset, contract)),
    (Bhei, BheiOpcode, "bhei", (ret)),
    (Bhsh, BhshOpcode, "bhsh", (addr, height)),
    (Burn, BurnOpcode, "burn", (coins)),
    (Call, CallOpcode, "call", (args_addr, coins, asset, gas)),
    (Cb, CbOpcode, "cb", (addr)),
    (Ccp, CcpOpcode, "ccp", (dst_addr, contract, src_addr, size)),
    (Croo, CrooOpcode, "croo", (addr, contract)),
    (Csiz, CsizOpcode, "csiz", (ret, contract)),
    (Ldc, LdcOpcode, "ldc", (contract, addr, size)),
    (Log, LogOpcode, "log", (reg_a, reg_b, reg_c, reg_d)),
    (Logd, LogdOpcode, "logd", (reg_a, reg_b, addr, size)),
    (Mint, MintOpcode, "mint", (coins)),
    (Retd, RetdOpcode, "retd", (addr, size)),
    (Rvrt, RvrtOpcode, "rvrt", (value)),
    (Srw, SrwOpcode, "srw", (ret, state_addr)),
    (Srwq, SrwqOpcode, "srwq", (addr, state_addr)),
    (Sww, SwwOpcode, "sww", (state_addr, value)),
    (Swwq, SwwqOpcode, "swwq", (state_addr, addr)),
    (Tr, TrOpcode, "tr", (contract, coins, asset)),
    (Tro, TroOpcode, "tro", (addr, output, coins, asset)),
    (Ecr, EcrOpcode, "ecr", (addr, sig, hash)),
    (K256, K256Opcode, "k256", (addr, data, size)),
    (S256, S256Opcode, "s256", (addr, data, size)),
    (Flag, FlagOpcode, "flag", (value)),
    (Gm, GmOpcode, "gm", (ret, op)),
);
