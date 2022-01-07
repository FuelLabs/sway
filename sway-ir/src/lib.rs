// For now it's easiest to just export absolutely everything to core_lang, we can refine the public
// API when it's closer to finished.

pub mod asm;
pub use asm::*;
pub mod block;
pub use block::*;
pub mod constant;
pub use constant::*;
pub mod context;
pub use context::*;
pub mod function;
pub use function::*;
pub mod instruction;
pub use instruction::*;
pub mod irtype;
pub use irtype::*;
pub mod module;
pub use module::*;
pub mod optimize;
pub use optimize::*;
pub mod parser;
pub use parser::*;
pub mod pointer;
pub use pointer::*;
pub mod printer;
pub use printer::*;
pub mod value;
pub use value::*;
pub mod verify;
pub use verify::*;
