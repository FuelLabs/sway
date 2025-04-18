library;

use ::trait::Trait;

pub fn uses_trait<T>(_a: T)
where
    T: Trait,
{}
