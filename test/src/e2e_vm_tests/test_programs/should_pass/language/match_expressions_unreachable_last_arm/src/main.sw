script;

struct Struct {
    x: bool,
    y: u32,
    z: (u32, u32, u32)
}

impl Struct {
    fn new() -> Self {
        Struct {
            x: false,
            y: 0,
            z: (0, 0, 0),
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
    E: u32,
}

fn main() -> () {
    let e = Enum::A;

    let _x = match e {
        Enum::A => 0,
        Enum::B => 0,
        Enum::C => 0,
        Enum::D => 0,
        Enum::E(_) => 0,
        Enum::E(1) => 0,
    };

    let s1 = Struct::new();

    let _x = match s1 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: false, y: 0, z } => z.0,
    };

    let s2 = Struct::new();

    let _x = match s2 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: true, y: 0, z: (0, 0, 0) } => 0,
    };

    let s3 = Struct::new();

    let _x = match s3 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: false, y: 0, z: (0, _, _) } => 0,
    };

    let s4 = Struct::new();

    let _x = match s4 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: true, y: 0, z: _ } => 0,
    };

    let s5 = Struct::new();

    let _x = match s5 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: false, y, z: _ } => y,
    };

    let s6 = Struct::new();

    let _x = match s6 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: true, y: 0, z: (0, 0, 0) } => 0,
    };
    
    // TODO: Once bug with Struct { .. } patterns and exhaustive match expressions is fixed, add this case as well.
    // let _x = match s_TODO {
    //     Struct { x: true, y, z } => y + z.0,
    //     Struct { x: false, y, z } => y + z.0,
    //     Struct { x:_, y: 0, z:_ } => 0,
    // };

    // TODO: Once bug with Struct { .. } patterns and exhaustive match expressions is fixed, add this case as well.
    // let _x = match s_TODO {
    //     Struct { x: true, y, z } => y + z.0,
    //     Struct { x: false, y, z } => y + z.0,
    //     Struct { x: _, y: 0, z: (_, _, _) } => 0,
    // };

    let t1 = (false, Enum::A, Struct::new(), 0u32);

    let _x = match t1 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (false, Enum::A, s, n) => n + s.y,
    };

    let t2 = (false, Enum::A, Struct::new(), 0u32);

    let _x = match t2 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (false, Enum::A, Struct { x: true, y, z:_ }, 0) => y,
    };

    let t3 = (false, Enum::A, Struct::new(), 0u32);

    let _x = match t3 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (true, _, Struct { x: true, y, z: _}, 0) => y,
    };

    let t4 = (false, Enum::A, Struct::new(), 0u32);

    let _x = match t4 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (true, _, _, 0) => 0,
    };

    let t5 = (false, Enum::A, Struct::new(), 0u32);

    let _x = match t5 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (true, _, Struct { x: _, y: _, z: (j, k, l)}, n) => {
            poke(j);
            poke(k);
            poke(l);
            n
        },
    };

    let t6 = (false, Enum::A, Struct::new(), 0u32);

    let _x = match t6 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (true, _, Struct { x: _, y: _, z: (_ , k, _)}, n) => {
            poke(k);
            n
        },
    };

    // TODO: Once bug with Struct { .. } patterns and exhaustive match expressions is fixed (https://github.com/FuelLabs/sway/issues/4950), add this case as well:
    // let _x = match t {
    //     (true, _, s, n) => n + s.y,
    //     (false, _, s, n) => n + s.y,
    //     (b, e, s, 0) => {
    //         poke(e);
    //         if b { s.y } else { 0 }
    //     },
    // };

    // TODO: Once bug with Struct { .. } patterns and exhaustive match expressions is fixed (https://github.com/FuelLabs/sway/issues/4950), add this case as well:
    // let _x = match t5 {
    //     (true, _, s, n) => n + s.y,
    //     (false, _, s, n) => n + s.y,
    //     (_, Enum::B, Struct { x: true, y: 0, z: (j, k, l)}, n) => {
    //         poke(j);
    //         poke(k);
    //         poke(l);
    //         n
    //     },
    // };

    poke(Enum::B);
    poke(Enum::C);
    poke(Enum::D);
    poke(Enum::E(0));
    poke(Struct::new().use_me());
}

fn poke<T>(_x: T) { }
