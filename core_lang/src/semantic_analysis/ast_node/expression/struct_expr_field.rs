use crate::semantic_analysis::TypedExpression;
use crate::Ident;

#[derive(Clone, Debug)]
pub(crate) struct TypedStructExpressionField<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) value: TypedExpression<'sc>,
}

impl TypedStructExpressionField<'_> {
    pub(crate) fn copy_types(&mut self) {
        self.value.copy_types();
    }
}
