use crate::priv_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct LitString {
    pub span: Span,
    pub parsed: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct LitChar {
    pub span: Span,
    pub parsed: char,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct LitInt {
    pub span: Span,
    pub parsed: BigUint,
    pub ty_opt: Option<(LitIntType, Span)>,
    /// True if this [LitInt] represents a `b256` hex literal
    /// in a manually generated lexed tree.
    ///
    /// `b256` hex literals are not explicitly modeled in the
    /// [Literal]. During parsing, they are parsed as [LitInt]
    /// with [LitInt::ty_opt] set to `None`.
    ///
    /// To properly render `b256` manually created hex literals,
    /// that are not backed by a [Span] in the source code,
    /// we need this additional information, to distinguish
    /// them from `u256` hex literals.
    pub is_generated_b256: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub enum LitIntType {
    U8,
    U16,
    U32,
    U64,
    U256,
    I8,
    I16,
    I32,
    I64,
    B256,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct LitBool {
    pub span: Span,
    pub kind: LitBoolType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub enum LitBoolType {
    True,
    False,
}

impl From<LitBoolType> for bool {
    fn from(item: LitBoolType) -> Self {
        match item {
            LitBoolType::True => true,
            LitBoolType::False => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Literal {
    String(LitString),
    Char(LitChar),
    Int(LitInt),
    Bool(LitBool),
}

impl Literal {
    /// Friendly type name string of the [Literal] used for various reportings.
    pub fn friendly_type_name(&self) -> &'static str {
        use Literal::*;
        match self {
            String(_) => "str",
            Char(_) => "char",
            Int(lit_int) => {
                if lit_int.is_generated_b256 {
                    "b256"
                } else {
                    lit_int
                        .ty_opt
                        .as_ref()
                        .map_or("numeric", |(ty, _)| match ty {
                            LitIntType::U8 => "u8",
                            LitIntType::U16 => "u16",
                            LitIntType::U32 => "u32",
                            LitIntType::U64 => "u64",
                            LitIntType::U256 => "u256",
                            LitIntType::I8 => "i8",
                            LitIntType::I16 => "i16",
                            LitIntType::I32 => "i32",
                            LitIntType::I64 => "i64",
                            LitIntType::B256 => "b256",
                        })
                }
            }
            Bool(_) => "bool",
        }
    }
}

impl Spanned for LitString {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for LitChar {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for LitInt {
    fn span(&self) -> Span {
        match &self.ty_opt {
            Some((_lit_int_ty, span)) => Span::join(self.span.clone(), span),
            None => self.span.clone(),
        }
    }
}

impl Spanned for LitBool {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for Literal {
    fn span(&self) -> Span {
        match self {
            Literal::String(lit_string) => lit_string.span(),
            Literal::Char(lit_char) => lit_char.span(),
            Literal::Int(lit_int) => lit_int.span(),
            Literal::Bool(lit_bool) => lit_bool.span(),
        }
    }
}
