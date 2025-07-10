use crate::{ParseResult, Parser};

use sway_ast::expr::op_code::*;
use sway_error::parser_error::ParseErrorKind;
use sway_types::{Ident, Spanned};

macro_rules! define_op_codes (
    ( $(($op_name:ident, $ty_name:ident, $s:literal, ($($arg_name:ident),*)),)* ) => {
        pub const OP_CODES: &'static [&'static str] = &[
            $($s),*
        ];

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
                    Err(parser.emit_error_with_span(ParseErrorKind::UnrecognizedOpCode {
                        known_op_codes: OP_CODES,
                    }, span))
                },
            }
        }
    };
);

define_op_codes!(
    /* Arithmetic/Logic (ALU) Instructions */
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
    (Srl, SrlOpcode, "srl", (ret, lhs, rhs)),
    (Srli, SrliOpcode, "srli", (ret, lhs, rhs)),
    (Sub, SubOpcode, "sub", (ret, lhs, rhs)),
    (Subi, SubiOpcode, "subi", (ret, lhs, rhs)),
    (Wqcm, WqcmOpcode, "wqcm", (ret, lhs, rhs, op_mode)),
    (Wqop, WqopOpcode, "wqop", (ret, lhs, rhs, op_mode)),
    (Wqml, WqmlOpcode, "wqml", (ret, lhs, rhs, indirect)),
    (Wqdv, WqdvOpcode, "wqdv", (ret, lhs, rhs, indirect)),
    (Wqmd, WqmdOpcode, "wqmd", (ret, lhs_a, lhs_b, rhs)),
    (Wqam, WqamOpcode, "wqam", (ret, lhs_a, lhs_b, rhs)),
    (Wqmm, WqmmOpcode, "wqmm", (ret, lhs_a, lhs_b, rhs)),
    (Xor, XorOpcode, "xor", (ret, lhs, rhs)),
    (Xori, XoriOpcode, "xori", (ret, lhs, rhs)),
    /* Control Flow Instructions */
    (Jmp, JmpOpcode, "jmp", (offset)),
    (Ji, JiOpcode, "ji", (offset)),
    (Jne, JneOpcode, "jne", (lhs, rhs, offset)),
    (Jnei, JneiOpcode, "jnei", (lhs, rhs, offset)),
    (Jnzi, JnziOpcode, "jnzi", (arg, offset)),
    (Ret, RetOpcode, "ret", (value)),
    /* Memory Instructions */
    (Aloc, AlocOpcode, "aloc", (size)),
    (Cfei, CfeiOpcode, "cfei", (size)),
    (Cfsi, CfsiOpcode, "cfsi", (size)),
    (Cfe, CfeOpcode, "cfe", (size)),
    (Cfs, CfsOpcode, "cfs", (size)),
    (Lb, LbOpcode, "lb", (ret, addr, offset)),
    (Lw, LwOpcode, "lw", (ret, addr, offset)),
    (Mcl, MclOpcode, "mcl", (addr, size)),
    (Mcli, McliOpcode, "mcli", (addr, size)),
    (Mcp, McpOpcode, "mcp", (dst_addr, src_addr, size)),
    (Mcpi, McpiOpcode, "mcpi", (dst_addr, src_addr, size)),
    (Meq, MeqOpcode, "meq", (ret, lhs_addr, rhs_addr, size)),
    (Sb, SbOpcode, "sb", (addr, value, offset)),
    (Sw, SwOpcode, "sw", (addr, value, offset)),
    /* Contract Instructions */
    (Bal, BalOpcode, "bal", (ret, asset, contract)),
    (Bhei, BheiOpcode, "bhei", (ret)),
    (Bhsh, BhshOpcode, "bhsh", (addr, height)),
    (Burn, BurnOpcode, "burn", (coins, sub_id)),
    (Call, CallOpcode, "call", (args_addr, coins, asset, gas)),
    (Cb, CbOpcode, "cb", (addr)),
    (Ccp, CcpOpcode, "ccp", (dst_addr, contract, src_addr, size)),
    (Croo, CrooOpcode, "croo", (addr, contract)),
    (Csiz, CsizOpcode, "csiz", (ret, contract)),
    (Bsiz, BsizOpcode, "bsiz", (ret, contract)),
    (Ldc, LdcOpcode, "ldc", (contract, addr, size, imm)),
    (Bldd, BlddOpcode, "bldd", (dst_ptr, addr, offset, len)),
    (Log, LogOpcode, "log", (reg_a, reg_b, reg_c, reg_d)),
    (Logd, LogdOpcode, "logd", (reg_a, reg_b, addr, size)),
    (Mint, MintOpcode, "mint", (coins, sub_id)),
    (Retd, RetdOpcode, "retd", (addr, size)),
    (Rvrt, RvrtOpcode, "rvrt", (value)),
    (Smo, SmoOpcode, "smo", (addr, len, output, coins)),
    (Scwq, ScwqOpcode, "scwq", (addr, is_set, len)),
    (Srw, SrwOpcode, "srw", (ret, is_set, state_addr)),
    (Srwq, SrwqOpcode, "srwq", (addr, is_set, state_addr, count)),
    (Sww, SwwOpcode, "sww", (state_addr, is_set, value)),
    (Swwq, SwwqOpcode, "swwq", (state_addr, is_set, addr, count)),
    (Time, TimeOpcode, "time", (ret, height)),
    (Tr, TrOpcode, "tr", (contract, coins, asset)),
    (Tro, TroOpcode, "tro", (addr, output, coins, asset)),
    (Gnse, GnseOpcode, "gnse", (addr, output, coins, asset)),
    /* Cryptographic Instructions */
    (Eck1, Eck1Opcode, "eck1", (addr, sig, hash)),
    (Ecr1, Ecr1Opcode, "ecr1", (addr, sig, hash)),
    (Ed19, Ed19Opcode, "ed19", (addr, sig, hash, len)),
    (K256, K256Opcode, "k256", (addr, data, size)),
    (S256, S256Opcode, "s256", (addr, data, size)),
    (
        ECOP,
        ECOPOpcode,
        "ecop",
        (dst_addr, curve, operation, src_addr)
    ),
    (
        EPAR,
        EPAROpcode,
        "epar",
        (ret, curve, groups_of_points, addr)
    ),
    /* Other Instructions */
    (Ecal, EcalOpcode, "ecal", (reg_a, reg_b, reg_c, reg_d)),
    (Flag, FlagOpcode, "flag", (value)),
    (Gm, GmOpcode, "gm", (ret, op)),
    (Gtf, GtfOpcode, "gtf", (ret, index, tx_field_id)),
    /* Non-VM Instructions */
    (Blob, BlobOpcode, "blob", (size)),
);
