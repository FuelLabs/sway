#[derive(Clone, PartialEq, Eq, Default)]
pub enum Mode {
    ImplAbiFn(sway_types::Ident),
    #[default]
    NonAbi,
}
