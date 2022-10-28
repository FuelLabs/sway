library helpers;

pub trait OutOfScopeGetter<D> {
    fn out_of_scope_get(self) -> D;
}
