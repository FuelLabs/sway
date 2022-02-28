use crate::priv_prelude::*;

macro_rules! define_op_code (
    ($ty_name:ident, $s:literal) => (
        #[derive(Clone, Debug)]
        pub struct $ty_name {
            span: Span,
        }

        impl $ty_name {
            pub fn span(&self) -> Span {
                self.span.clone()
            }
        }

        /*
        impl Peek for $ty_name {
            fn peek(peeker: Peeker<'_>) -> Option<$ty_name> {
                let ident = peeker.peek_ident().ok()?;
                if ident.as_str() == $s {
                    Some($ty_name { span: ident.span() })
                } else {
                    None
                }
            }
        }

        impl Parse for $ty_name {
            fn parse(parser: &mut Parser) -> ParseResult<$ty_name> {
                match parser.take() {
                    Some(value) => Ok(value),
                    None => Err(parser.emit_error(format!("expected `{}`", stringify!($ty_name)))),
                }
            }
        }
        */
    );
);

macro_rules! define_op_codes (
    ($(($op_name:ident, $ty_name:ident, $s:literal),)*) => {
        $(define_op_code!($ty_name, $s);)*

        #[derive(Clone, Debug)]
        pub enum OpCode {
            $($op_name($ty_name),)*
        }

        impl Peek for OpCode {
            fn peek(peeker: Peeker<'_>) -> Option<OpCode> {
                let ident = peeker.peek_ident().ok()?;
                match ident.as_str() {
                    $($s => {
                        Some(OpCode::$op_name($ty_name { span: ident.span() }))
                    },)*
                    _ => None,
                }
            }
        }

        /*
        impl Parse for OpCode {
            fn parse(parser: Parser) -> ParseResult<OpCode> {
                let ident = peeker.peek_ident().ok()?;
                match ident.as_str() {
                    $($s => {
                        Ok(OpCode::$op_name($ty_name { span: ident.span() }))
                    },)*
                    _ => None,
                }
            }
        }
        */
    };
);

define_op_codes!(
    (Add, AddOpcode, "add"),
    (Addi, AddiOpcode, "addi"),
    (And, AndOpcode, "and"),
    (Andi, AndiOpcode, "andi"),
    (Div, DivOpcode, "div"),
    (Divi, DiviOpcode, "divi"),
    (Eq, EqOpcode, "eq"),
    (Exp, ExpOpcode, "exp"),
    (Expi, ExpiOpcode, "expi"),
    (Gt, GtOpcode, "gt"),
    (Lt, LtOpcode, "lt"),
    (Mlog, MlogOpcode, "mlog"),
    (Mod, ModOpcode, "mod"),
    (Modi, ModiOpcode, "modi"),
    (Move, MoveOpcode, "move"),
    (Mroo, MrooOpcode, "mroo"),
    (Mul, MulOpcode, "mul"),
    (Muli, MuliOpcode, "muli"),
    (Noop, NoopOpcode, "noop"),
    (Not, NotOpcode, "not"),
    (Or, OrOpcode, "or"),
    (Ori, OriOpcode, "ori"),
    (Sll, SllOpcode, "sll"),
    (Slli, SlliOpcode, "slli"),
    (Srl, SrlOpcode, "srl"),
    (Srli, SrliOpcode, "srli"),
    (Sub, SubOpcode, "sub"),
    (Subi, SubiOpcode, "subi"),
    (Xor, XorOpcode, "xor"),
    (Xori, XoriOpcode, "xori"),
    (Cimv, CimvOpcode, "cimv"),
    (Ctmv, CtmvOpcode, "ctmv"),
    (Ji, JiOpcode, "ji"),
    (Jnei, JneiOpcode, "jnei"),
    (Ret, RetOpcode, "ret"),
    (Aloc, AlocOpcode, "aloc"),
    (Cfei, CfeiOpcode, "cfei"),
    (Cfsi, CfsiOpcode, "cfsi"),
    (Lb, LbOpcode, "lb"),
    (Lw, LwOpcode, "lw"),
    (Mcl, MclOpcode, "mcl"),
    (Mcli, McliOpcode, "mcli"),
    (Mcp, McpOpcode, "mcp"),
    (Mcpi, McpiOpcode, "mcpi"),
    (Meq, MeqOpcode, "meq"),
    (Sb, SbOpcode, "sb"),
    (Sw, SwOpcode, "sw"),
    (Bal, BalOpcode, "bal"),
    (Bhei, BheiOpcode, "bhei"),
    (Bhsh, BhshOpcode, "bhsh"),
    (Burn, BurnOpcode, "burn"),
    (Call, CallOpcode, "call"),
    (Cb, CbOpcode, "cb"),
    (Ccp, CcpOpcode, "ccp"),
    (Croo, CrooOpcode, "croo"),
    (Csiz, CsizOpcode, "csiz"),
    (Ldc, LdcOpcode, "ldc"),
    (Log, LogOpcode, "log"),
    (Logd, LogdOpcode, "logd"),
    (Mint, MintOpcode, "mint"),
    (Retd, RetdOpcode, "retd"),
    (Rvrt, RvrtOpcode, "rvrt"),
    (Sldc, SldcOpcode, "sldc"),
    (Srw, SrwOpcode, "srw"),
    (Srwq, SrwqOpcode, "srwq"),
    (Sww, SwwOpcode, "sww"),
    (Swwq, SwwqOpcode, "swwq"),
    (Tr, TrOpcode, "tr"),
    (Tro, TroOpcode, "tro"),
    (Ecr, EcrOpcode, "ecr"),
    (K256, K256Opcode, "k256"),
    (S256, S256Opcode, "s256"),
    (Xil, XilOpcode, "xil"),
    (Xis, XisOpcode, "xis"),
    (Xol, XolOpcode, "xol"),
    (Xos, XosOpcode, "xos"),
    (Xwl, XwlOpcode, "xwl"),
    (Xws, XwsOpcode, "xws"),
    (Flag, FlagOpcode, "flag"),
    (Gm, GmOpcode, "gm"),
);

