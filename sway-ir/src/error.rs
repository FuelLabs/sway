#[derive(Debug)]
pub enum IrError {
    FunctionLocalClobbered(String, String),
    InvalidMetadatum,
    MismatchedReturnTypes(String),
    MisplacedTerminator(String),
    MissingBlock(String),
    MissingTerminator(String),
    NonUniquePhiLabels,
    ParseFailure(String, String),
    ValueNotFound(String),
}

use std::fmt;

impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            IrError::FunctionLocalClobbered(fn_str, var_str) => write!(
                f,
                "Local storage for function {fn_str} already has an entry for variable {var_str}"
            ),
            IrError::InvalidMetadatum => write!(f, "Unable to convert from invalid metadatum."),
            IrError::MismatchedReturnTypes(fn_str) => write!(
                f,
                "Function {fn_str} return type must match its RET instructions."
            ),
            IrError::MisplacedTerminator(blk_str) => {
                write!(f, "Block {blk_str} has a misplaced terminator.")
            }
            IrError::MissingBlock(blk_str) => write!(f, "Unable to find block {blk_str}."),
            IrError::MissingTerminator(blk_str) => {
                write!(f, "Block {blk_str} is missing its terminator.")
            }
            IrError::NonUniquePhiLabels => write!(f, "PHI must have unique block labels."),
            IrError::ParseFailure(expecting, found) => {
                write!(f, "Parse failure: expecting '{expecting}', found '{found}'")
            }
            IrError::ValueNotFound(reason) => {
                write!(f, "Invalid value: {reason}")
            }
        }
    }
}
