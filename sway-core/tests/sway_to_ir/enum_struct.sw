script;

enum ABC {
    A: (),
    B: XYZ,
    C: (),
}

struct XYZ {
    x: b256,
    y: bool,
    z: u64,
}

fn main() {
    ABC::B(XYZ {
        x: 0x0001010101010101000101010101010100010101010101010001010101010101,
        y: true,
        z: 53,
    });
}
