use fuel_asm::Word;
use fuel_tx::{crypto, ContractAddress, Hash};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

pub type Id = [u8; Hash::size_of()];
pub type Contract = [u8; ContractAddress::size_of()];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

/// Based on https://llvm.org/docs/CoverageMappingFormat.html#source-code-range
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
    /// Beginning of the code range
    pub start: Position,
    /// End of the code range (inclusive)
    pub end: Position,
}

impl Range {
    pub const fn is_valid(&self) -> bool {
        self.start.line < self.end.line
            || self.start.line == self.end.line && self.start.col <= self.end.col
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Instruction {
    /// Relative to the `$is`
    pub pc: Word,
    /// Code range that translates to this point
    pub range: Range,
    /// Exit from the current scope?
    pub exit: bool,
}

impl Instruction {
    pub fn to_bytes(&self) -> [u8; 41] {
        let mut bytes = [0u8; 41];

        // Always convert to `u64` to avoid architectural variants of the bytes representation that
        // could lead to arch-dependent unique IDs
        bytes[..8].copy_from_slice(&(self.pc as u64).to_be_bytes());
        bytes[8..16].copy_from_slice(&(self.range.start.line as u64).to_be_bytes());
        bytes[16..24].copy_from_slice(&(self.range.start.col as u64).to_be_bytes());
        bytes[24..32].copy_from_slice(&(self.range.end.line as u64).to_be_bytes());
        bytes[32..40].copy_from_slice(&(self.range.end.col as u64).to_be_bytes());
        bytes[40] = self.exit as u8;

        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Source {
    /// Absolute path to the source file
    path: PathBuf,
}

impl<T> From<T> for Source
where
    T: Into<PathBuf>,
{
    fn from(path: T) -> Self {
        Self { path: path.into() }
    }
}

impl AsRef<PathBuf> for Source {
    fn as_ref(&self) -> &PathBuf {
        &self.path
    }
}

impl AsMut<PathBuf> for Source {
    fn as_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }
}

/// Contract call stack frame representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallFrame {
    /// Deterministic representation of the frame
    id: Id,
    /// Contract that contains the bytecodes of this frame. Currently only scripts are supported
    contract: Contract,
    /// Sway source code that compiles to this frame
    source: Source,
    /// Range of code that represents this frame
    range: Range,
    /// Set of instructions that describes this frame
    program: Vec<Instruction>,
}

impl CallFrame {
    pub fn new(
        contract: ContractAddress,
        source: Source,
        range: Range,
        program: Vec<Instruction>,
    ) -> io::Result<Self> {
        let contract = Contract::from(contract);

        if !source.path.as_path().is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The source path must be absolute!",
            ));
        }

        if !source.path.as_path().is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The source path must be a valid Sway source file!",
            ));
        }

        if !source.path.as_path().exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "The source path must point to an existing file!",
            ));
        }

        if !range.is_valid() || program.iter().any(|i| !i.range.is_valid()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The provided source range is inconsistent!",
            ));
        }

        let mut repr: Vec<u8> = source
            .path
            .as_path()
            .to_str()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to get the string representation of the path!",
                )
            })?
            .as_bytes()
            .iter()
            .chain(contract.iter())
            .copied()
            .collect();

        // IntoIter for 1.52.1 compat
        program
            .iter()
            .map(Instruction::to_bytes)
            .for_each(|b| repr.extend(&b));

        let id = *crypto::hash(repr.as_slice());

        Ok(Self {
            id,
            contract,
            source,
            range,
            program,
        })
    }

    pub const fn id(&self) -> &Id {
        &self.id
    }

    pub const fn source(&self) -> &Source {
        &self.source
    }

    pub const fn range(&self) -> &Range {
        &self.range
    }

    pub const fn contract_raw(&self) -> &Contract {
        &self.contract
    }

    pub fn program(&self) -> &[Instruction] {
        self.program.as_slice()
    }

    pub fn contract(&self) -> ContractAddress {
        self.contract.into()
    }
}
