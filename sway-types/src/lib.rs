use fuel_asm::Word;
use fuel_crypto::Hasher;
use fuel_tx::{Bytes32, ContractId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{io, iter, slice};

pub mod ident;
pub use ident::*;

pub mod span;
pub use span::*;

pub mod state;

pub type Id = [u8; Bytes32::LEN];
pub type Contract = [u8; ContractId::LEN];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

/// Based on `<https://llvm.org/docs/CoverageMappingFormat.html#source-code-range>`
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

    pub fn bytes<'a>(iter: impl Iterator<Item = &'a Self>) -> Vec<u8> {
        // Need to return owned bytes because flatten is not supported by 1.53 for arrays bigger
        // than 32 bytes
        iter.map(Self::to_bytes)
            .fold::<Vec<u8>, _>(vec![], |mut v, b| {
                v.extend(&b);

                v
            })
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigTimeConstant {
    pub r#type: String,
    pub value: String,
}
impl AsRef<PathBuf> for Source {
    fn as_ref(&self) -> &PathBuf {
        &self.path
    }
}

impl AsRef<Path> for Source {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl AsMut<PathBuf> for Source {
    fn as_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }
}

impl Source {
    pub fn bytes(&self) -> io::Result<slice::Iter<'_, u8>> {
        Ok(self
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
            .iter())
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
        contract: ContractId,
        source: Source,
        range: Range,
        program: Vec<Instruction>,
    ) -> io::Result<Self> {
        Context::validate_source(&source)?;
        Context::validate_range(iter::once(&range).chain(program.iter().map(|p| &p.range)))?;

        let contract = Contract::from(contract);

        let id = Context::id_from_repr(
            Instruction::bytes(program.iter())
                .iter()
                .chain(contract.iter())
                .chain(source.bytes()?),
        );

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

    pub fn program(&self) -> &[Instruction] {
        self.program.as_slice()
    }

    pub fn contract(&self) -> ContractId {
        self.contract.into()
    }
}

/// Transaction script interpreter representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionScript {
    /// Deterministic representation of the script
    id: Id,
    /// Sway source code that compiles to this script
    source: Source,
    /// Range of code that represents this script
    range: Range,
    /// Set of instructions that describes this script
    program: Vec<Instruction>,
}

impl TransactionScript {
    pub fn new(source: Source, range: Range, program: Vec<Instruction>) -> io::Result<Self> {
        Context::validate_source(&source)?;
        Context::validate_range(iter::once(&range).chain(program.iter().map(|p| &p.range)))?;

        let id = Context::id_from_repr(
            Instruction::bytes(program.iter())
                .iter()
                .chain(source.bytes()?),
        );

        Ok(Self {
            id,
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

    pub fn program(&self) -> &[Instruction] {
        self.program.as_slice()
    }
}

// Representation of a debug context to be mapped from a sway source and consumed by the DAP-sway
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Context {
    CallFrame(CallFrame),
    TransactionScript(TransactionScript),
}

impl From<CallFrame> for Context {
    fn from(frame: CallFrame) -> Self {
        Self::CallFrame(frame)
    }
}

impl From<TransactionScript> for Context {
    fn from(script: TransactionScript) -> Self {
        Self::TransactionScript(script)
    }
}

impl Context {
    pub fn validate_source<P>(path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        if !path.as_ref().is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The source path must be absolute!",
            ));
        }

        if !path.as_ref().is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The source path must be a valid Sway source file!",
            ));
        }

        if !path.as_ref().exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "The source path must point to an existing file!",
            ));
        }

        Ok(())
    }

    pub fn validate_range<'a>(mut range: impl Iterator<Item = &'a Range>) -> io::Result<()> {
        if !range.any(|r| !r.is_valid()) {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The provided source range is inconsistent!",
            ))
        } else {
            Ok(())
        }
    }

    pub fn id_from_repr<'a>(bytes: impl Iterator<Item = &'a u8>) -> Id {
        let bytes: Vec<u8> = bytes.copied().collect();

        *Hasher::hash(bytes.as_slice())
    }

    pub const fn id(&self) -> &Id {
        match self {
            Self::CallFrame(t) => t.id(),
            Self::TransactionScript(t) => t.id(),
        }
    }

    pub const fn source(&self) -> &Source {
        match self {
            Self::CallFrame(t) => t.source(),
            Self::TransactionScript(t) => t.source(),
        }
    }

    pub const fn range(&self) -> &Range {
        match self {
            Self::CallFrame(t) => t.range(),
            Self::TransactionScript(t) => t.range(),
        }
    }

    pub fn program(&self) -> &[Instruction] {
        match self {
            Self::CallFrame(t) => t.program(),
            Self::TransactionScript(t) => t.program(),
        }
    }

    pub fn contract(&self) -> Option<ContractId> {
        match self {
            Self::CallFrame(t) => Some(t.contract()),
            _ => None,
        }
    }
}

/// TODO: The types `Function` and `Property` below are copied from `fuels-types` except for the
/// `typeArguments` field of `Property`. Switch back to using fuels-types
/// directly when the `typeArguments` field is added there
///
/// Fuel ABI representation in JSON, originally specified here:
///
/// https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/abi.md
///
/// This type may be used by compilers (e.g. Sway) and related tooling to convert an ABI
/// representation into native Rust structs and vice-versa.

pub type JsonABI = Vec<Function>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    #[serde(rename = "type")]
    pub type_field: String,
    pub inputs: Vec<Property>,
    pub name: String,
    pub outputs: Vec<Property>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub components: Option<Vec<Property>>, // Used for custom types
    pub type_arguments: Option<Vec<Property>>, // Used for generic types. Not yet supported in fuels-rs.
}
