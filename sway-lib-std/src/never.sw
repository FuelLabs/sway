library never;

use core::ops::*;

pub enum Never {}

impl Not for Never {
    fn not(self) -> Self {
        self
    }
}