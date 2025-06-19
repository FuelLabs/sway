library;

pub enum State { Uninitialized: () }

abi SRC5 {
    fn owner() -> State;
}

impl PartialEq for State {
    fn eq(self, other: Self) -> bool {
        true
    }
}

impl Eq for State {}
