use super::IntegerBits;
use crate::Ident;
/// [ResolvedType] refers to a fully qualified type that has been looked up in the namespace.
/// Type symbols are ambiguous in the beginning of compilation, as any custom symbol could be
/// an enum, struct, or generic type name. This enum is similar to [TypeInfo], except it lacks
/// the capability to be `TypeInfo::Custom`, i.e., pending this resolution of whether it is generic or a
/// known type. This allows us to ensure structurally that no unresolved types bleed into the
/// syntax tree.
#[derive(Debug, Clone)]
pub enum ResolvedType<'sc> {
    String,
    UnsignedInteger(IntegerBits),
    Boolean,
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Generic {
        name: Ident<'sc>,
    },
    Unit,
    SelfType,
    Byte,
    Byte32,
    Struct {
        name: Ident<'sc>,
    },
    Enum {
        name: Ident<'sc>,
    },
    // used for recovering from errors in the ast
    ErrorRecovery,
}

impl<'sc> ResolvedType<'sc> {
    pub(crate) fn friendly_type_str(&self) -> String {
        use ResolvedType::*;
        match self {
            String => "String".into(),
            UnsignedInteger(bits) => {
                use IntegerBits::*;
                match bits {
                    Eight => "u8",
                    Sixteen => "u16",
                    ThirtyTwo => "u32",
                    SixtyFour => "u64",
                    OneTwentyEight => "u128",
                }
                .into()
            }
            Boolean => "bool".into(),
            Generic { name } => format!("generic {}", name.primary_name),
            Unit => "()".into(),
            SelfType => "Self".into(),
            Byte => "byte".into(),
            Byte32 => "byte32".into(),
            Struct {
                name: Ident { primary_name, .. },
                ..
            } => format!("struct {}", primary_name),
            Enum {
                name: Ident { primary_name, .. },
                ..
            } => format!("enum {}", primary_name),
            ErrorRecovery => "\"unknown due to error\"".into(),
        }
    }
}
