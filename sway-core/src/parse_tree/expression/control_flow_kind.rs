
/// An expression that affects control flow
#[derive(Debug, Clone)]
pub(crate) enum ControlFlowKind {
    Break,
    Continue
}