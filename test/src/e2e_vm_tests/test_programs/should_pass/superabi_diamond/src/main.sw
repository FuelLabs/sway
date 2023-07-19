contract;

///         Top
///      /       \
///    Left     Right
///      \       /
///        Bottom

abi Top {
    fn top();
}

abi Left : Top {
    fn left();
}

abi Right : Top {
    fn right();
}

abi Bottom : Left + Right {
    fn bottom();
}

// This should be allowed in principle, because
// Left::top() and Right::top() actually refer
// to the same method Top::top().
// We forbid it temporarily because it's easier to
// implement it this way before we have proper infrastructure
impl Top for Contract {
    fn top() { }
}

impl Left for Contract {
    fn left() { }
}

impl Right for Contract {
    fn right() { }
}

impl Bottom for Contract {
    fn bottom() { }
}
