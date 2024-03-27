#[derive(Clone, Debug, PartialEq, Eq, Hash, deepsize::DeepSizeOf)]
pub enum LazyOp {
    And,
    Or,
}
