contract;

///         Top
///      /       \
///    Left     Right
///      \       /
///        Bottom

abi Top
{
    // no interface methods
}
{
    fn top() {}
}

abi Left : Top
{
    // no interface methods
}
{
    fn left() {}
}

abi Right : Top
{
    // no interface methods
}
{
    fn right() {}
}

abi Bottom : Left + Right
{
    // no interface methods
}
{
    fn bottom() {}
}

impl Top for Contract { }

impl Left for Contract { }

impl Right for Contract { }

impl Bottom for Contract { }
