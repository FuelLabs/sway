use super::{lexed::LexedProgram, parsed::ParseProgram, ty::TyProgram};

/// Contains the lexed, parsed, and typed compilation stages of a program.
pub struct Programs {
    pub lexed: LexedProgram,
    pub parsed: ParseProgram,
    pub typed: Option<TyProgram>,
}

impl Programs {
    pub fn new(lexed: LexedProgram, parsed: ParseProgram, typed: Option<TyProgram>) -> Programs {
        Programs {
            lexed,
            parsed,
            typed,
        }
    }
}
