use super::InstructionSet;
/// Represents an ASM set which has had register allocation, jump elimination, and optimization
/// applied to it
pub enum FinalizedAsm<'sc> {
    ContractAbi,
    ScriptMain {
        program_section: InstructionSet<'sc>,
    },
    PredicateMain {
        program_section: InstructionSet<'sc>,
    },
    // Libraries do not generate any asm.
    Library,
}

impl<'sc> FinalizedAsm<'sc> {
    fn to_bytecode(&self) -> Vec<u8> {
        todo!()
    }
}
