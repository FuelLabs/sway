use crate::priv_prelude::*;

macro_rules! define_op_code (
    ($ty_name:ident, $s:literal) => (
        #[derive(Clone, Debug, Serialize)]
        pub struct $ty_name {
            span: Span,
        }

        impl Spanned for $ty_name {
            fn span(&self) -> Span {
                self.span.clone()
            }
        }
    );
);

macro_rules! op_code_ty (
    (reg) => { Ident };
    (imm) => { AsmImmediate };
);

macro_rules! push_register_arg_idents (
    ($vec_name:ident, ()) => {};
    ($vec_name:ident, ($arg_name_head:ident: reg, $($arg_name:ident: $arg_ty:tt,)*)) => {
        $vec_name.push($arg_name_head.clone());
        push_register_arg_idents!($vec_name, ($($arg_name: $arg_ty,)*))
    };
    ($vec_name:ident, ($arg_name_head:ident: imm, $($arg_name:ident: $arg_ty:tt,)*)) => {
        let _ = $arg_name_head;
        push_register_arg_idents!($vec_name, ($($arg_name: $arg_ty,)*))
    };
);

macro_rules! push_immediate_idents (
    ($vec_name:ident, ()) => {};
    ($vec_name:ident, ($arg_name_head:ident: reg, $($arg_name:ident: $arg_ty:tt,)*)) => {
        let _ = $arg_name_head;
        push_immediate_idents!($vec_name, ($($arg_name: $arg_ty,)*))
    };
    ($vec_name:ident, ($arg_name_head:ident: imm, $($arg_name:ident: $arg_ty:tt,)*)) => {
        $vec_name.push(Ident::new($arg_name_head.span()));
        push_immediate_idents!($vec_name, ($($arg_name: $arg_ty,)*))
    };
);

macro_rules! ignore_remaining (
    () => {};
    ($arg_name_head:ident: $arg_ty_head:tt, $($arg_name:ident: $arg_ty:tt,)*) => {{
        let _ = $arg_name_head;
        ignore_remaining!($($arg_name: $arg_ty,)*);
    }};
);

macro_rules! immediate_ident_opt (
    () => {
        None
    };
    ($arg_name_head:ident: reg, $($arg_name:ident: $arg_ty:tt,)*) => {{
        let _ = $arg_name_head;
        immediate_ident_opt!($($arg_name: $arg_ty,)*)
    }};
    ($arg_name_head:ident: imm, $($arg_name:ident: $arg_ty:tt,)*) => {{
        ignore_remaining!($($arg_name: $arg_ty,)*);
        Some(Ident::new($arg_name_head.span()))
    }};
);

macro_rules! get_span (
    ($start:expr, ()) => { $start };
    ($start:expr, ($arg_name:ident,)) => {
        Span::join($start, &$arg_name.span())
    };
    ($start:expr, ($arg_name_head:ident, $($arg_name:ident,)*)) => {{
        let _ = $arg_name_head;
        get_span!($start, ($($arg_name,)*))
    }};
);

/// A specific instruction.
pub trait Inst {
    /// The instruction's literal in source code.
    const LIT: &'static str;

    /// Arguments to the instruction.
    type Args;

    fn instruction(ident: Ident, args: Self::Args) -> Instruction;
}

macro_rules! define_op_codes (
    ($(($op_name:ident, $ty_name:ident, $s:literal, ($($arg_name:ident: $arg_ty:tt),*)),)*) => {
        $(
            define_op_code!($ty_name, $s);

            impl Inst for $ty_name {
                const LIT: &'static str = $s;
                type Args = ($(op_code_ty!($arg_ty),)*);

                fn instruction(ident: Ident, ($($arg_name,)*): Self::Args) -> Instruction {
                    Instruction::$op_name {
                        token: $ty_name { span: ident.span().clone() },
                        $($arg_name,)*
                    }
                }
            }
        )*

        #[derive(Clone, Debug, Serialize)]
        pub enum Instruction {
            $($op_name {
                token: $ty_name,
                $($arg_name: op_code_ty!($arg_ty),)*
            },)*
        }

        impl Instruction {
            pub fn op_code_ident(&self) -> Ident {
                match self {
                    $(Instruction::$op_name { token, .. } => {
                        Ident::new(token.span())
                    },)*
                }
            }

            pub fn op_code_as_str(&self) -> &'static str {
                match self {
                    $(Instruction::$op_name { .. } => {
                        $s
                    },)*
                }
            }

            #[allow(clippy::vec_init_then_push)]
            pub fn register_arg_idents(&self) -> Vec<Ident> {
                match self {
                    $(Instruction::$op_name { $($arg_name,)* .. } => {
                        #[allow(unused_mut)]
                        let mut ret = Vec::new();
                        push_register_arg_idents!(ret, ($($arg_name: $arg_ty,)*));
                        ret
                    },)*
                }
            }

            pub fn immediate_ident_opt(&self) -> Option<Ident> {
                match self {
                    $(Instruction::$op_name { $($arg_name,)* .. } => {
                        immediate_ident_opt!($($arg_name: $arg_ty,)*)
                    },)*
                }
            }

            #[allow(clippy::vec_init_then_push)]
            pub fn immediate_idents(&self) -> Vec<Ident> {
                match self {
                    $(Instruction::$op_name { $($arg_name,)* .. } => {
                        #[allow(unused_mut)]
                        let mut ret = Vec::new();
                        push_immediate_idents!(ret, ($($arg_name: $arg_ty,)*));
                        ret
                    },)*
                }
            }

        }

        impl Spanned for Instruction {
            fn span(&self) -> Span {
                match self {
                    $(Instruction::$op_name { token, $($arg_name,)* } => {
                        get_span!(token.span(), ($($arg_name,)*))
                    },)*
                }
            }
        }
    };
);

define_op_codes!(
    /* Arithmetic/Logic (ALU) Instructions */
    (Add, AddOpcode, "add", (ret: reg, lhs: reg, rhs: reg)),
    (Addi, AddiOpcode, "addi", (ret: reg, lhs: reg, rhs: imm)),
    (And, AndOpcode, "and", (ret: reg, lhs: reg, rhs: reg)),
    (Andi, AndiOpcode, "andi", (ret: reg, lhs: reg, rhs: imm)),
    (Div, DivOpcode, "div", (ret: reg, lhs: reg, rhs: reg)),
    (Divi, DiviOpcode, "divi", (ret: reg, lhs: reg, rhs: imm)),
    (Eq, EqOpcode, "eq", (ret: reg, lhs: reg, rhs: reg)),
    (Exp, ExpOpcode, "exp", (ret: reg, base: reg, power: reg)),
    (Expi, ExpiOpcode, "expi", (ret: reg, base: reg, power: imm)),
    (Gt, GtOpcode, "gt", (ret: reg, lhs: reg, rhs: reg)),
    (Lt, LtOpcode, "lt", (ret: reg, lhs: reg, rhs: reg)),
    (Mlog, MlogOpcode, "mlog", (ret: reg, arg: reg, base: reg)),
    (Mod, ModOpcode, "mod", (ret: reg, lhs: reg, rhs: reg)),
    (Modi, ModiOpcode, "modi", (ret: reg, lhs: reg, rhs: imm)),
    (Move, MoveOpcode, "move", (ret: reg, from: reg)),
    (Movi, MoviOpcode, "movi", (ret: reg, arg: imm)),
    (Mroo, MrooOpcode, "mroo", (ret: reg, arg: reg, root: reg)),
    (Mul, MulOpcode, "mul", (ret: reg, lhs: reg, rhs: reg)),
    (Muli, MuliOpcode, "muli", (ret: reg, lhs: reg, rhs: imm)),
    (Noop, NoopOpcode, "noop", ()),
    (Not, NotOpcode, "not", (ret: reg, arg: reg)),
    (Or, OrOpcode, "or", (ret: reg, lhs: reg, rhs: reg)),
    (Ori, OriOpcode, "ori", (ret: reg, lhs: reg, rhs: imm)),
    (Sll, SllOpcode, "sll", (ret: reg, lhs: reg, rhs: reg)),
    (Slli, SlliOpcode, "slli", (ret: reg, lhs: reg, rhs: imm)),
    (Srl, SrlOpcode, "srl", (ret: reg, lhs: reg, rhs: reg)),
    (Srli, SrliOpcode, "srli", (ret: reg, lhs: reg, rhs: imm)),
    (Sub, SubOpcode, "sub", (ret: reg, lhs: reg, rhs: reg)),
    (Subi, SubiOpcode, "subi", (ret: reg, lhs: reg, rhs: imm)),
    (Wqcm, WqcmOpcode, "wqcm", (ret: reg, lhs: reg, rhs: reg, op_mode: imm)),
    (Wqop, WqopOpcode, "wqop", (ret: reg, lhs: reg, rhs: reg, op_mode: imm)),
    (Wqml, WqmlOpcode, "wqml", (ret: reg, lhs: reg, rhs: reg, indirect: imm)),
    (Wqdv, WqdvOpcode, "wqdv", (ret: reg, lhs: reg, rhs: reg, indirect: imm)),
    (Wqmd, WqmdOpcode, "wqmd", (ret: reg, lhs_a: reg, lhs_b: reg, rhs: reg)),
    (Wqam, WqamOpcode, "wqam", (ret: reg, lhs_a: reg, lhs_b: reg, rhs: reg)),
    (Wqmm, WqmmOpcode, "wqmm", (ret: reg, lhs_a: reg, lhs_b: reg, rhs: reg)),
    (Xor, XorOpcode, "xor", (ret: reg, lhs: reg, rhs: reg)),
    (Xori, XoriOpcode, "xori", (ret: reg, lhs: reg, rhs: imm)),
    /* Control Flow Instructions */
    (Jmp, JmpOpcode, "jmp", (offset: reg)),
    (Ji, JiOpcode, "ji", (offset: imm)),
    (Jne, JneOpcode, "jne", (lhs: reg, rhs: reg, offset: reg)),
    (Jnei, JneiOpcode, "jnei", (lhs: reg, rhs: reg, offset: imm)),
    (Jnzi, JnziOpcode, "jnzi", (arg: reg, offset: imm)),
    (Ret, RetOpcode, "ret", (value: reg)),
    /* Memory Instructions */
    (Aloc, AlocOpcode, "aloc", (size: reg)),
    (Cfei, CfeiOpcode, "cfei", (size: imm)),
    (Cfsi, CfsiOpcode, "cfsi", (size: imm)),
    (Cfe, CfeOpcode, "cfe", (size: reg)),
    (Cfs, CfsOpcode, "cfs", (size: reg)),
    (Lb, LbOpcode, "lb", (ret: reg, addr: reg, offset: imm)),
    (Lw, LwOpcode, "lw", (ret: reg, addr: reg, offset: imm)),
    (Mcl, MclOpcode, "mcl", (addr: reg, size: reg)),
    (Mcli, McliOpcode, "mcli", (addr: reg, size: imm)),
    (
        Mcp,
        McpOpcode,
        "mcp",
        (dst_addr: reg, src_addr: reg, size: reg)
    ),
    (
        Mcpi,
        McpiOpcode,
        "mcpi",
        (dst_addr: reg, src_addr: reg, size: imm)
    ),
    (
        Meq,
        MeqOpcode,
        "meq",
        (ret: reg, lhs_addr: reg, rhs_addr: reg, size: reg)
    ),
    (Sb, SbOpcode, "sb", (addr: reg, value: reg, offset: imm)),
    (Sw, SwOpcode, "sw", (addr: reg, value: reg, offset: imm)),
    /* Contract Instructions */
    (Bal, BalOpcode, "bal", (ret: reg, asset: reg, contract: reg)),
    (Bhei, BheiOpcode, "bhei", (ret: reg)),
    (Bhsh, BhshOpcode, "bhsh", (addr: reg, height: reg)),
    (Burn, BurnOpcode, "burn", (coins: reg, sub_id: reg)),
    (
        Call,
        CallOpcode,
        "call",
        (args_addr: reg, coins: reg, asset: reg, gas: reg)
    ),
    (Cb, CbOpcode, "cb", (addr: reg)),
    (
        Ccp,
        CcpOpcode,
        "ccp",
        (dst_addr: reg, contract: reg, src_addr: reg, size: reg)
    ),
    (Croo, CrooOpcode, "croo", (addr: reg, contract: reg)),
    (Csiz, CsizOpcode, "csiz", (ret: reg, contract: reg)),
    (Bsiz, BsizOpcode, "bsiz", (ret: reg, contract: reg)),
    (Ldc, LdcOpcode, "ldc", (contract: reg, addr: reg, size: reg, mode: imm)),
    (Bldd, BlddOpcode, "bldd", (dst_ptr: reg, addr: reg, offset: reg, len: reg)),
    (
        Log,
        LogOpcode,
        "log",
        (reg_a: reg, reg_b: reg, reg_c: reg, reg_d: reg)
    ),
    (
        Logd,
        LogdOpcode,
        "logd",
        (reg_a: reg, reg_b: reg, addr: reg, size: reg)
    ),
    (Mint, MintOpcode, "mint", (coins: reg, sub_id: reg)),
    (Retd, RetdOpcode, "retd", (addr: reg, size: reg)),
    (Rvrt, RvrtOpcode, "rvrt", (value: reg)),
    (
        Smo,
        SmoOpcode,
        "smo",
        (addr: reg, len: reg, output: reg, coins: reg)
    ),
    (Scwq, ScwqOpcode, "scwq", (addr: reg, is_set: reg, len: reg)),
    (
        Srw,
        SrwOpcode,
        "srw",
        (ret: reg, is_set: reg, state_addr: reg)
    ),
    (
        Srwq,
        SrwqOpcode,
        "srwq",
        (addr: reg, is_set: reg, state_addr: reg, count: reg)
    ),
    (
        Sww,
        SwwOpcode,
        "sww",
        (state_addr: reg, is_set: reg, value: reg)
    ),
    (
        Swwq,
        SwwqOpcode,
        "swwq",
        (state_addr: reg, is_set: reg, addr: reg, count: reg)
    ),
    (Time, TimeOpcode, "time", (ret: reg, height: reg)),
    (Tr, TrOpcode, "tr", (contract: reg, coins: reg, asset: reg)),
    (
        Tro,
        TroOpcode,
        "tro",
        (addr: reg, output: reg, coins: reg, asset: reg)
    ),
    (
        Gnse,
        GnseOpcode,
        "gnse",
        (status: reg, start_key: reg, dst_key_value: reg, direction: imm)
    ),
    /* Cryptographic Instructions */
    (Eck1, Eck1Opcode, "eck1", (addr: reg, sig: reg, hash: reg)),
    (Ecr1, Ecr1Opcode, "ecr1", (addr: reg, sig: reg, hash: reg)),
    (Ed19, Ed19Opcode, "ed19", (addr: reg, sig: reg, hash: reg, len: reg)),
    (K256, K256Opcode, "k256", (addr: reg, data: reg, size: reg)),
    (S256, S256Opcode, "s256", (addr: reg, data: reg, size: reg)),
    (ECOP, ECOPOpcode, "ecop", (dst_addr: reg, curve: reg, operation: reg, src_addr: reg)),
    (EPAR, EPAROpcode, "epar", (ret: reg, curve: reg, groups_of_points: reg, addr: reg)),
    /* Other Instructions */
    (Ecal, EcalOpcode, "ecal", (reg_a: reg, reg_b: reg, reg_c: reg, reg_d: reg)),
    (Flag, FlagOpcode, "flag", (value: reg)),
    (Gm, GmOpcode, "gm", (ret: reg, op: imm)),
    (
        Gtf,
        GtfOpcode,
        "gtf",
        (ret: reg, index: reg, tx_field_id: imm)
    ),
    /* Non-VM Instructions */
    (Blob, BlobOpcode, "blob", (size: imm)),
);
