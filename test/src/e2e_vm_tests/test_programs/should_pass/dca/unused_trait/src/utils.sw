library utils;

dep r#trait;
use r#trait::Trait;

pub fn uses_trait<T>(a: T) where T: Trait {

}
