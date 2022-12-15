use std::fmt;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Intrinsic {
    GetStorageKey,
    IsReferenceType,
    SizeOfType,
    SizeOfVal,
    Eq,
    Gtf,
    AddrOf,
    StateLoadWord,
    StateStoreWord,
    StateLoadQuad,
    StateStoreQuad,
    Log,
    Add,
    Sub,
    Mul,
    Div,
    Revert,
    PtrAdd,
    PtrSub,
    Smo,
}

impl fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Intrinsic::GetStorageKey => "get_storage_key",
            Intrinsic::IsReferenceType => "is_reference_type",
            Intrinsic::SizeOfType => "size_of",
            Intrinsic::SizeOfVal => "size_of_val",
            Intrinsic::Eq => "eq",
            Intrinsic::Gtf => "gtf",
            Intrinsic::AddrOf => "addr_of",
            Intrinsic::StateLoadWord => "state_load_word",
            Intrinsic::StateStoreWord => "state_store_word",
            Intrinsic::StateLoadQuad => "state_load_quad",
            Intrinsic::StateStoreQuad => "state_store_quad",
            Intrinsic::Log => "log",
            Intrinsic::Add => "add",
            Intrinsic::Sub => "sub",
            Intrinsic::Mul => "mul",
            Intrinsic::Div => "div",
            Intrinsic::Revert => "revert",
            Intrinsic::PtrAdd => "ptr_add",
            Intrinsic::PtrSub => "ptr_sub",
            Intrinsic::Smo => "smo",
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
            "__gtf" => Gtf,
            "__addr_of" => AddrOf,
            "__state_load_word" => StateLoadWord,
            "__state_store_word" => StateStoreWord,
            "__state_load_quad" => StateLoadQuad,
            "__state_store_quad" => StateStoreQuad,
            "__log" => Log,
            "__add" => Add,
            "__sub" => Sub,
            "__mul" => Mul,
            "__div" => Div,
            "__revert" => Revert,
            "__ptr_add" => PtrAdd,
            "__ptr_sub" => PtrSub,
            "__smo" => Smo,
            _ => return None,
        })
    }
}
