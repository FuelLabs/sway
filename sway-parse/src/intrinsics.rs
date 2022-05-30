#[derive(Eq, PartialEq)]
pub enum Intrinsic {
    GenerateUid,
    IsReferenceType,
    SizeOf,
    SizeOfVal,
}

impl Intrinsic {
    pub fn try_from_str(raw: &str) -> Option<Intrinsic> {
        use Intrinsic::*;
        Some(match raw {
            "__generate_uid" => GenerateUid,
            "__is_reference_type" => IsReferenceType,
            "__size_of" => SizeOf,
            "__size_of_val" => SizeOfVal,
            _ => return None,
        })
    }
}
