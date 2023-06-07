#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    ImplAbiFn,
    #[default]
    NonAbi,
}
