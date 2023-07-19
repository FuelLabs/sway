#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AbiMode {
    ImplAbiFn,
    #[default]
    NonAbi,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ConstShadowingMode {
    Sequential,
    #[default]
    ItemStyle,
}
