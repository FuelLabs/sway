script;

struct Struct {
    x: bool,
    y: u64,
    z: (u64, u64, u64)
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
    E: u64,
}

fn main() -> () {
    let e1 = Enum::A;

    let _x = match e1 {
        Enum::A => 0,
        Enum::B => 0,
        Enum::C => 0,
        Enum::D => 0,
        Enum::E(_) => 1,
        _ => 0,
    };

    let e2 = Enum::A;

    let _x = match e2 {
        Enum::A => 0,
        Enum::B => 0,
        Enum::C => 0,
        Enum::D => 0,
        Enum::E(_) => 2,
        x => { 
            poke(x);
            0
        } ,
    };

    let s1 = Struct::new();

    let _x = match s1 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        _ => 0,
    };

    let s2 = Struct::new();

    let _x = match s2 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.1,
        x => {
            poke(x);
            0
        },
    };

    let s3 = Struct::new();

    let _x = match s3 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x, y, z } => if x { y } else { z.0 },
    };

    let s4 = Struct::new();

    let _x = match s4 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x:_, y:_, z:_ } => 0,
    };

    let s5 = Struct::new();

    let _x = match s5 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: a, y: b, z: c } => if a { b } else { c.0 },
    };

    let s6 = Struct::new();

    let _x = match s6 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: a, y: b, z: (j, k, l) } => if a { b } else { j + k + l },
    };

    let s7 = Struct::new();

    let _x = match s7 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: a, y: b, z: (_, _, _) } => if a { b } else { 0 },
    };

    // TODO: Once bug with Struct { .. } patterns and exhaustive match expressions is fixed, add this case as well.
    // let _x = match s_TODO {
    //     Struct { x: true, y, z } => y + z.0,
    //     Struct { x: false, y, z } => y + z.0,
    //     Struct { x, .. } => if x { 1 } else { 0 },
    // };

    // TODO: Once bug with Struct { .. } patterns and exhaustive match expressions is fixed, add this case as well.
    // let _x = match s_TODO {
    //     Struct { x: true, y, z } => y + z.0,
    //     Struct { x: false, y, z } => y + z.0,
    //     Struct { .. } => 0,
    // };
    
    let t1 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t1 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        _ => 0,
    };

    let t2 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t2 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.z.0,
        x => x.3,
    };

    let t3 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t3 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (b, e, s, n) => {
            poke(e);
            if b { s.y } else { n }
        },
    };

    let t4 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t4 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.z.0,
        (_, _, _, _) => 0,
    };

    let t5 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t5 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (_, _, _, n) => n,
    };

    let t6 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t6 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (_, _, Struct { x, y, z: (j , k, l)}, n) => {
            poke(x);
            poke(y);
            poke(j);
            poke(k);
            poke(l);
            n
        },
    };

    let t7 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t7 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (_, _, Struct { x: _, y: _, z: (_ , k, _)}, n) => {
            poke(k);
            n
        },
    };

    // TODO: Add this test as well once reachability issues are fixed: https://github.com/FuelLabs/sway/issues/5116
    // let or1 = true;

    // let _x = match or1 {
    //     true => 0,
    //     false => 0,
    //     true | false | _ => 0,
    // };

    poke(Enum::B);
    poke(Enum::C);
    poke(Enum::D);
    poke(Enum::E(0));
    poke(Struct::new().use_me());
}

fn poke<T>(_x: T) { }
