/// An object that is a template for copies from the template.
///
/// This is predominantly used with [SubstList](crate::type_system::SubstList)
/// and [TyDeclaration](crate::language::ty::TyDeclaration). The various
/// variants of [TyDeclaration](crate::language::ty::TyDeclaration) contain
/// fields `subst_list: Template<SubstList>`. This type indicates that the
/// [SubstList](crate::type_system::SubstList) contained in this field is simply
/// a template for usages of the declaration declared in that particular
/// [TyDeclaration](crate::language::ty::TyDeclaration) node.
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

    #[allow(dead_code)]
    pub(crate) fn into_inner(self) -> T {
        self.0
    }

    #[allow(dead_code)]
    pub(crate) fn fresh_copy(&self) -> T {
        self.0.clone()
    }
}
