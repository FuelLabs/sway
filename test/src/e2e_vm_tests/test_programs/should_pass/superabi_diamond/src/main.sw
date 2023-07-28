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
