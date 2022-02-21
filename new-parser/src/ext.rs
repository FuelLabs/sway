use crate::priv_prelude::*;

#[extension_trait]
pub impl InfallibleExt for Infallible {
    fn unreachable<T>(self) -> T {
        match self {}
    }
}

