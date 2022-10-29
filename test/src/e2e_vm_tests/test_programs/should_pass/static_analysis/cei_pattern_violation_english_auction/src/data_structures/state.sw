library state;

pub enum State {
    /// The state at which the auction is no longer accepting bids.
    Closed: (),
    /// The state where bids may be placed on an auction.
    Open: (),
}

impl core::ops::Eq for State {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (State::Open, State::Open) => true,
            (State::Closed, State::Closed) => true,
            _ => false,
        }
    }
}
