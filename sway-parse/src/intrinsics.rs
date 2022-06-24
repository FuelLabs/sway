#[derive(Eq, PartialEq)]
pub enum Intrinsic {
    GetStorageKey,
    IsReferenceType,
    SizeOf,
    SizeOfVal,
    Eq,
}

impl Intrinsic {
    pub fn try_from_str(raw: &str) -> Option<Intrinsic> {
        use Intrinsic::*;
        Some(match raw {
            "__get_storage_key" => GetStorageKey,
            "__is_reference_type" => IsReferenceType,
            "__size_of" => SizeOf,
            "__size_of_val" => SizeOfVal,
            "__eq" => Eq,
            _ => return None,
        })
    }
}
