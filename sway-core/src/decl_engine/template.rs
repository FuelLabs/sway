use crate::{CreateCopy, Engines};

/// An object that is a template for copies from the template.
///
/// This is predominantly used with [SubstList](crate::type_system::SubstList)
/// and [TyDecl](crate::language::ty::TyDecl). The various variants of
/// [TyDecl](crate::language::ty::TyDecl) contain fields
/// `subst_list: Template<SubstList>`. This type indicates that the
/// [SubstList](crate::type_system::SubstList) contained in this field is simply
/// a template for usages of the declaration declared in that particular
/// [TyDecl](crate::language::ty::TyDecl) node.
#[derive(Clone, Debug)]
pub struct Template<T>(T)
where
    T: Clone;

impl<T> Template<T>
where
    T: Clone,
{
    pub(crate) fn new(value: T) -> Template<T> {
        Template(value)
    }

    pub(crate) fn inner(&self) -> &T {
        &self.0
    }
}

impl<T> CreateCopy<T> for Template<T>
where
    T: Clone + CreateCopy<T>,
{
    fn scoped_copy(&self, engines: Engines<'_>) -> T {
        self.0.scoped_copy(engines)
    }

    fn unscoped_copy(&self) -> T {
        self.0.unscoped_copy()
    }
}
