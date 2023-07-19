#[derive(Clone, PartialEq, Eq, Default)]
pub enum AbiMode {
    ImplAbiFn(sway_types::Ident),
    #[default]
    NonAbi,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ConstShadowingMode {
    Sequential,
    #[default]
    ItemStyle,
}
