use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AbstractOp {
    // corresponds to "Ensure this value is on top of the stack"
    AccessValue(sway_ir::Value),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DirectOp {
    Exec(super::ProcedureName),

    ProcedureDecl {
        name: super::ProcedureName,
        number_of_locals: u32,
    },
    Begin,
    End,
    If {
        condition: Vec<MidenAsmOp>,
        true_branch: Vec<MidenAsmOp>,
        else_branch: Vec<MidenAsmOp>,
    },
    Push(MidenStackValue),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MidenStackValue {
    Unit,
    Number(u64),
    Bool(bool),
}

impl Display for MidenStackValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MidenStackValue::Unit => "0".into(),
                MidenStackValue::Number(x) => x.to_string(),
                MidenStackValue::Bool(b) => if *b { "1" } else { "0" }.into(),
            },
        )
    }
}
impl From<bool> for MidenStackValue {
    fn from(value: bool) -> Self {
        MidenStackValue::Bool(value)
    }
}

impl From<u64> for MidenStackValue {
    fn from(value: u64) -> Self {
        MidenStackValue::Number(value)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MidenAsmOp {
    DirectOp(DirectOp),
    /// This is an abstract Op. Meaning, this doesn't correspond to an
    /// individual miden vm asm op.     
    AbstractOp(AbstractOp),
    /*
    Assert,
    AssertEq,
    Assertz,
    Add,
    AddImm(Felt),
    Sub,
    SubImm(Felt),
    Mul,
    MulImm(Felt),
    Div,
    DivImm(Felt),
    Neg,
    Inv,
    Pow2,
    Exp,
    ExpImm(Felt),
    ExpBitLength(u8),
    Not,
    And,
    Or,
    Xor,
    Eq,
    EqImm(Felt),
    Neq,
    NeqImm(Felt),
    Eqw,
    Lt,
    Lte,
    Gt,
    Gte,

    // ----- u32 manipulation ---------------------------------------------------------------
    U32Test,
    U32TestW,
    U32Assert,
    U32Assert2,
    U32AssertW,
    U32Split,
    U32Cast,
    U32CheckedAdd,
    U32CheckedAddImm(u32),
    U32WrappingAdd,
    U32WrappingAddImm(u32),
    U32OverflowingAdd,
    U32OverflowingAddImm(u32),
    U32OverflowingAdd3,
    U32WrappingAdd3,
    U32CheckedSub,
    U32CheckedSubImm(u32),
    U32WrappingSub,
    U32WrappingSubImm(u32),
    U32OverflowingSub,
    U32OverflowingSubImm(u32),
    U32CheckedMul,
    U32CheckedMulImm(u32),
    U32WrappingMul,
    U32WrappingMulImm(u32),
    U32OverflowingMul,
    U32OverflowingMulImm(u32),
    U32OverflowingMadd,
    U32WrappingMadd,
    U32CheckedDiv,
    U32CheckedDivImm(u32),
    U32UncheckedDiv,
    U32UncheckedDivImm(u32),
    U32CheckedMod,
    U32CheckedModImm(u32),
    U32UncheckedMod,
    U32UncheckedModImm(u32),
    U32CheckedDivMod,
    U32CheckedDivModImm(u32),
    U32UncheckedDivMod,
    U32UncheckedDivModImm(u32),
    U32CheckedAnd,
    U32CheckedOr,
    U32CheckedXor,
    U32CheckedNot,
    U32CheckedShr,
    U32CheckedShrImm(u8),
    U32UncheckedShr,
    U32UncheckedShrImm(u8),
    U32CheckedShl,
    U32CheckedShlImm(u8),
    U32UncheckedShl,
    U32UncheckedShlImm(u8),
    U32CheckedRotr,
    U32CheckedRotrImm(u8),
    U32UncheckedRotr,
    U32UncheckedRotrImm(u8),
    U32CheckedRotl,
    U32CheckedRotlImm(u8),
    U32UncheckedRotl,
    U32UncheckedRotlImm(u8),
    U32CheckedEq,
    U32CheckedEqImm(u32),
    U32CheckedNeq,
    U32CheckedNeqImm(u32),
    U32CheckedLt,
    U32UncheckedLt,
    U32CheckedLte,
    U32UncheckedLte,
    U32CheckedGt,
    U32UncheckedGt,
    U32CheckedGte,
    U32UncheckedGte,
    U32CheckedMin,
    U32UncheckedMin,
    U32CheckedMax,
    U32UncheckedMax,

    // ----- stack manipulation ---------------------------------------------------------------
    Drop,
    DropW,
    PadW,
    Dup0,
    Dup1,
    Dup2,
    Dup3,
    Dup4,
    Dup5,
    Dup6,
    Dup7,
    Dup8,
    Dup9,
    Dup10,
    Dup11,
    Dup12,
    Dup13,
    Dup14,
    Dup15,
    DupW0,
    DupW1,
    DupW2,
    DupW3,
    Swap1,
    Swap2,
    Swap3,
    Swap4,
    Swap5,
    Swap6,
    Swap7,
    Swap8,
    Swap9,
    Swap10,
    Swap11,
    Swap12,
    Swap13,
    Swap14,
    Swap15,
    SwapW1,
    SwapW2,
    SwapW3,
    SwapDw,
    MovUp2,
    MovUp3,
    MovUp4,
    MovUp5,
    MovUp6,
    MovUp7,
    MovUp8,
    MovUp9,
    MovUp10,
    MovUp11,
    MovUp12,
    MovUp13,
    MovUp14,
    MovUp15,
    MovUpW2,
    MovUpW3,
    MovDn2,
    MovDn3,
    MovDn4,
    MovDn5,
    MovDn6,
    MovDn7,
    MovDn8,
    MovDn9,
    MovDn10,
    MovDn11,
    MovDn12,
    MovDn13,
    MovDn14,
    MovDn15,
    MovDnW2,
    MovDnW3,
    CSwap,
    CSwapW,
    CDrop,
    CDropW,

    // ----- input / output operations --------------------------------------------------------
    PushConstants(Vec<Felt>),
    Locaddr(u16),
    Sdepth,
    Caller,

    MemLoad,
    MemLoadImm(u32),
    MemLoadW,
    MemLoadWImm(u32),
    LocLoad(u16),
    LocLoadW(u16),

    MemStore,
    MemStoreImm(u32),
    LocStore(u16),
    MemStoreW,
    MemStoreWImm(u32),
    LocStoreW(u16),

    MemStream,
    AdvPipe,

    AdvPush(u8),
    AdvLoadW,

    AdvU64Div,
    AdvKeyval,
    AdvMem(u32, u32),

    // ----- cryptographic operations ---------------------------------------------------------
    RpHash,
    RpPerm,
    MTreeGet,
    MTreeSet,
    MTreeCwm,

    // ----- exec / call ----------------------------------------------------------------------
    ExecLocal(u16),
    ExecImported(ProcedureId),
    CallLocal(u16),
    CallImported(ProcedureId),
    SysCall(ProcedureId),
    */
}

impl DirectOp {
    pub(crate) fn begin() -> DirectOp {
        DirectOp::Begin
    }

    pub(crate) fn end() -> DirectOp {
        DirectOp::End
    }
    pub(crate) fn procedure_decl(name: String, number_of_locals: u32) -> DirectOp {
        DirectOp::ProcedureDecl {
            name,
            number_of_locals,
        }
    }
}
impl<T> Push<T> for DirectOp
where
    T: Into<MidenStackValue>,
{
    fn push(val: T) -> Self {
        let num = val.into();
        DirectOp::Push(num)
    }
}

pub trait Push<T> {
    fn push(val: T) -> Self;
}

impl MidenAsmOp {
    pub(crate) fn access_value(value: sway_ir::Value) -> MidenAsmOp {
        MidenAsmOp::AbstractOp(AbstractOp::AccessValue(value))
    }

    pub(crate) fn procedure_decl(name: String, number_of_locals: u32) -> MidenAsmOp {
        MidenAsmOp::DirectOp(DirectOp::procedure_decl(name, number_of_locals))
    }
}

impl Display for DirectOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DirectOp::*;
        f.write_str(&match self {
            Begin => "begin".into(),
            End => "end".into(),
            Exec(name) => format!("exec.{name}"),
            If {
                condition: _,
                true_branch,
                else_branch,
            } => format!(
                "if.true\n{}{}",
                indented(true_branch),
                if else_branch.is_empty() {
                    "".into()
                } else {
                    format!("else\n{}\nend", indented(else_branch))
                }
            ),

            ProcedureDecl {
                name,
                number_of_locals,
            } => format!("proc.{name}.{number_of_locals}"),
            Push(val) => format!("push.{val}"),
        })
    }
}

impl Display for AbstractOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AbstractOp::*;
        f.write_str(&match self {
            AccessValue(ref v) => format!("~access {v:?}"),
        })
    }
}
impl Display for MidenAsmOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MidenAsmOp::*;
        let text = match self {
            DirectOp(op) => format!("{op}"),
            AbstractOp(op) => format!("{op}"),
        };
        f.write_str(&text)
    }
}

/// indents some ops by 1 tab
fn indented(raw: &[MidenAsmOp]) -> String {
    let ops = raw.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    ops.join("\n\t")
}
