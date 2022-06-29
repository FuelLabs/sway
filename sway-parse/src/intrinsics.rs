use std::fmt;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Intrinsic {
    GetStorageKey,
    IsReferenceType,
    SizeOfType,
    SizeOfVal,
    Eq,
}

impl fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Intrinsic::GetStorageKey => "get_storage_key",
            Intrinsic::IsReferenceType => "is_reference_type",
            Intrinsic::SizeOfType => "size_of",
            Intrinsic::SizeOfVal => "size_of_val",
            Intrinsic::Eq => "eq",
        };
        write!(f, "{}", s)
    }
}

impl Intrinsic {
    pub fn try_from_str(raw: &str) -> Option<Intrinsic> {
        use Intrinsic::*;
        Some(match raw {
            "__get_storage_key" => GetStorageKey,
            "__is_reference_type" => IsReferenceType,
            "__size_of" => SizeOfType,
            "__size_of_val" => SizeOfVal,
            "__eq" => Eq,
            _ => return None,
        })
    }
}
