library never;

use ::ops::Not;

pub enum Never {}

impl Not for Never {
    fn not(self) -> Self {
        self
    }
}