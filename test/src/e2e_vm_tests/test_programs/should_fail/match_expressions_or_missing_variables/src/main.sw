script;

struct Struct {
    x: bool,
    y: u64,
    z: u8,
}

impl Struct {
    fn new() -> Self {
        Self {
            x: false,
            y: 0,
            z: 0,
        }
    }

    fn use_me(self) -> () {
        poke(self.x);
        poke(self.y);
        poke(self.z);
    }
}

fn main() {
    let s1 = Struct::new();

    let _x = match s1 {
        Struct { x, y, z } | Struct { x: y, y: x, z: a } => if y { 0 } else { x },
    };

    let s2 = Struct::new();

    let _x = match s2 {
        Struct { x, y, z } | Struct { x: y, y: x, z: a } | Struct { x: y, y: x, z: b } => if y { 0 } else { x },
    };

    let s3 = Struct::new();

    let _x = match s3 {
        Struct { x, y, z } | true | Struct { x: y, y: x, z: b } => if y { 0 } else { x },
    };

    let s4 = Struct::new();

    let _x = match s4 {
        true | Struct { x, y, z } | Struct { x: y, y: x, z: b } => if y { 0 } else { x },
    };

    let s5 = Struct::new();

    let _x = match s5 {
        Struct { x, y, z } | Struct { x: y, y: x, z: b } | true => if y { 0 } else { x },
    };

    let s6 = Struct::new();

    let _x = match s6 {
        Struct { x, y, z } | Struct { x, y: b, z: a } | Struct { x, y: b, z: a } | Struct { x, y: b, z: a } => if x { 0 } else { y },
    };

    poke(Struct::new().use_me());
}

fn poke<T>(_x: T) { }
