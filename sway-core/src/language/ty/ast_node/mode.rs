#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    ImplAbiFn,
    NonAbi,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::NonAbi
    }
}
