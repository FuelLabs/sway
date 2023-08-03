use sway_error::handler::ErrorEmitted;

use super::{lexed::LexedProgram, parsed::ParseProgram, ty::TyProgram};

/// Contains the lexed, parsed, and typed compilation stages of a program.
pub struct Programs {
    pub lexed: LexedProgram,
    pub parsed: ParseProgram,
    pub typed: Result<TyProgram, ErrorEmitted>,
}

impl Programs {
    pub fn new(
        lexed: LexedProgram,
        parsed: ParseProgram,
        typed: Result<TyProgram, ErrorEmitted>,
    ) -> Programs {
        Programs {
            lexed,
            parsed,
            typed,
        }
    }
}
