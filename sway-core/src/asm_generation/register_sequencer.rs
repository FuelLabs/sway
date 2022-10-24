use crate::asm_lang::{Label, VirtualRegister};
/// The [RegisterSequencer] is basically an iterator over integers -- it distributes unique ids in
/// the form of integers while ASM is being generated to ensure a monotonically increasing unique
/// register Id for every virtual register that is used.
#[derive(Default)]
pub(crate) struct RegisterSequencer {
    next_register:   usize,
    next_jump_label: usize,
}

impl RegisterSequencer {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    /// Choosing to not use the iterator trait, because this iterator goes on forever and thusly
    /// does not need to return an `Option<Item>`.
    pub(crate) fn next(&mut self) -> VirtualRegister {
        let next_val = self.next_register;
        self.next_register += 1;
        VirtualRegister::Virtual(next_val.to_string())
    }
    pub(crate) fn get_label(&mut self) -> Label {
        let next_val = self.next_jump_label;
        self.next_jump_label += 1;
        Label(next_val)
    }
}
