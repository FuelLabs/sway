library helpers;

pub trait OutOfScopeGetter<D> {
    fn get(self) -> D;
}
