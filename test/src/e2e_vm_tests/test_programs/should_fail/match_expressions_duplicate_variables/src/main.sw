script;

struct Struct {
    x: u64,
    y: u64,
    z: u64,
}

impl Struct {
    fn new() -> Self {
        Struct {
            x: 0,
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

enum Enum {
    A: (),
    B: (),
    C: (),
    D: (),
    E: (u64, Struct),
}

fn main() {
    let s1 = Struct::new();

    let _x = match s1 {
        Struct { x: 0, y: a_s1, z: a_s1 } => a_s1,
        _ => 0,
    };

    let s2 = Struct::new();
            
    let _x = match s2 {
        Struct { x, y: x, z: x } => x,
    };

    let s3 = Struct::new();
            
    let _x = match s3 {
        Struct { x, y: x, .. } => x,
    };
            
    let s4 = (Struct::new(), Struct::new());
            
    let _x = match s4 {
        (Struct { y, .. }, Struct { y, .. }) => y,
    };

    let t1 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t1 {
        (true, a_t1, a_t1, a_t1) => a_t1,
        (a_t1, a_t1, a_t1, a_t1) => a_t1,
    };
            
    let t2 = (false, Enum::A, Struct::new(), 0u64, 0u64);

    let _x = match t2 {
        (true, a_t2, Struct { x, .. }, a_t2, x) => a_t2,
        _ => 0,
    };
            
    let t3 = (false, Enum::A, Struct::new(), 0u64, 0u64);

    let _x = match t3 {
        (x, a_t3, Struct { .. }, a_t2, x) => a_t2,
    };

    let or1 = Struct::new();
            
    let _x = match or1 {
        Struct { x, y: x, z: x } | Struct { x, y: x, z: x } => x,
    };
            
    let or2 = (Struct::new(), Struct::new());
            
    let _x = match or2 {
        (Struct { x, .. } | Struct { x, .. }, Struct { x, .. } | Struct { x, .. }) => if x == 0 { 0 } else { 1 },
    };
            
    let or3 = Struct::new();
            
    let _x = match or3 {
        Struct { x: y | y | y, y, .. } | Struct { x: y | y | y, y, .. } => y,
    };

    let or4 = Enum::A;
            
    let _x = match or4 {
        Enum::E((1u64 | 2u64 | 3u64, Struct { x: 1 | 2, ..})) => 0,
        Enum::E((y, Struct { x: y | y | y, .. })) => 0,
        _ => 0,
    };

    let or5 = (0, Struct::new(), 0);

    let _x = match or5 {
        (x, Struct { x, .. } | Struct { x, .. }, y) => y,
        _ => 0,
    };

    let or6 = Struct::new();
            
    let _x = match or6 {
        Struct { x, y: x, .. } | Struct { x, .. } => x,
    };

    let or7 = Struct::new();
            
    let _x = match or7 {
        Struct { y, .. } | Struct { y, x: y, .. } => x,
    };

    poke(Enum::A);
    poke(Enum::B);
    poke(Enum::C);
    poke(Enum::D);
    poke(Enum::E((0, Struct::new())));
    poke(Struct::new().use_me());
}

fn poke<T>(_x: T) { }
