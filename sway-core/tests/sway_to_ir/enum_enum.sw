script;

enum ABC {
    A: (),
    B: XYZ,
    C: (),
}

enum XYZ {
    X: (),
    Y: bool,
    Z: (),
}

fn main() {
    ABC::B(XYZ::X);
}
