use super::{TypedCodeBlock, TypedExpression};

#[derive(Clone, Debug)]
pub(crate) struct TypedIfStatement<'sc> {
    pub(crate) condition: TypedExpression<'sc>,
    pub(crate) then: TypedCodeBlock<'sc>,
    pub(crate) r#else: Option<TypedCodeBlock<'sc>>
}

impl<'sc> TypedIfStatement<'sc> {
    pub(crate) fn pretty_print(&self) -> String {
        format!("if statement on {}", self.condition.pretty_print())
    }
}
