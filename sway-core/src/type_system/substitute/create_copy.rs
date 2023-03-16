use crate::Engines;

pub(crate) trait CreateCopy<T> {
    fn scoped_copy(&self, engines: Engines<'_>) -> T;
    fn unscoped_copy(&self) -> T;
}
