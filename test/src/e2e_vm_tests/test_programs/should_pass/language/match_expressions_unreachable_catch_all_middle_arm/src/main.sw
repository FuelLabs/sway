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
    let e0 = Enum::A;

    let _x = match e0 {
        _ => 0,
    };

    let e1_1 = Enum::A;

    let _x = match e1_1 {
        _ => 0,
        Enum::A => 0,
        Enum::B => 0,
        Enum::C => 0,
        Enum::D => 0,
        Enum::E(_) => 0,
    };

    let e1_2 = Enum::A;

    let _x = match e1_2 {
        Enum::A => 0,
        Enum::B => 0,
        _ => 0,
        Enum::C => 0,
        Enum::D => 0,
        _ => 0,
        Enum::E(_) => 0,
        _ => 0,
        _ => 0,
        _ => 0,
    };

    let e2_1 = Enum::A;

    let _x = match e2_1 {
        x => { 
            poke(x);
            0
        },
        Enum::A => 0,
        Enum::B => 0,
        Enum::C => 0,
        Enum::D => 0,
        Enum::E(_) => 0,
    };

    let e2_2 = Enum::A;

    let _x = match e2_2 {
        Enum::A => 0,
        Enum::B => 0,
        x => { 
            poke(x);
            0
        },
        Enum::C => 0,
        Enum::D => 0,
        x => { 
            poke(x);
            0
        },
        Enum::E(_) => 0,
        x => { 
            poke(x);
            0
        },
        _ => 0,
    };

    let e2_3 = Enum::A;

    let _x = match e2_3 {
        Enum::A => 0,
        Enum::B => 0,
        Enum::C => 0,
        Enum::D => 0,
        x => { 
            poke(x);
            0
        },
        Enum::E(_) => 0,
    };

    let e2_4 = Enum::A;

    let _x = match e2_4 {
        Enum::A => 0,
        Enum::B => 0,
        Enum::C => 0,
        Enum::D => 0,
        y => { 
            poke(y);
            0
        },
        _ => 0,
    };

    let s1 = Struct::new();

    let _x = match s1 {
        Struct { x: true, y, z } => y + z.0,
        _ => 0,
        Struct { x: false, y:0, z } => z.0,
        _ => 0,
        Struct { x: false, y, z } => y + z.0,
        _ => 0,
        _ => 0,
    };

    let s2 = Struct::new();

    let _x = match s2 {
        Struct { x: true, y, z } => y + z.0,
        x => {
            poke(x);
            0
        },
        Struct { x: false, y:0, z } => z.0,
        x => {
            poke(x);
            0
        },
        Struct { x: false, y, z } => y + z.0,
        x => {
            poke(x);
            0
        },
        x => {
            poke(x);
            0
        },
        _ => 0,
    };

    let s3 = Struct::new();

    let _x = match s3 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y:0, z } => z.0,
        Struct { x, y, z } => if x { y } else { z.0 },
        Struct { x: false, y, z } => y + z.0,
        Struct { x, y, z } => if x { y } else { z.0 },
        Struct { x, y, z } => if x { y } else { z.0 },
        _ => 0,
    };

    let s4 = Struct::new();

    let _x = match s4 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x:_, y:_, z:_ } => 0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x:_, y:_, z:_ } => 0,
        Struct { x:_, y:_, z:_ } => 0,
        _ => 0,
    };

    let s5 = Struct::new();

    let _x = match s5 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: a, y: b, z: c } => if a { b } else { c.0 },
        Struct { x: false, y, z } => y + z.0,
        Struct { x: a, y: b, z: c } => if a { b } else { c.0 },
        Struct { x: a, y: b, z: c } => if a { b } else { c.0 },
        _ => 0,
    };

    let s6 = Struct::new();

    let _x = match s6 {
        Struct { x: true, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: a, y: b, z: (j, k, l) } => if a { b } else { j + k + l },
        Struct { x: false, y, z } => y + z.0,
        Struct { x: false, y, z } => y + z.0,
        Struct { x: a, y: b, z: (j, k, l) } => if a { b } else { j + k + l },
        Struct { x: a, y: b, z: (j, k, l) } => if a { b } else { j + k + l },
        _ => 0,
    };

    let s7 = Struct::new();

    let _x = match s7 {
        Struct { x: a, y: b, z: (_, _, _) } => if a { b } else { 0 },
        Struct { x: true, y, z } => y + z.0,
        Struct { x: a, y: b, z: (_, _, _) } => if a { b } else { 0 },
        Struct { x: false, y, z } => y + z.0,
        Struct { x: a, y: b, z: (_, _, _) } => if a { b } else { 0 },
        Struct { x: a, y: b, z: (_, _, _) } => if a { b } else { 0 },
        _ => 0,
    };

    // TODO: Once bug with Struct { .. } patterns and exhaustive match expressions is fixed, add this case as well.
    // let _x = match s {
    //     Struct { x: true, y, z } => y + z.0,
    //     Struct { x: false, y, z } => y + z.0,
    //     Struct { x, .. } => if x { 1 } else { 0 },
    //     TODO
    // };

    // TODO: Once bug with Struct { .. } patterns and exhaustive match expressions is fixed, add this case as well.
    // let _x = match s {
    //     Struct { x: true, y, z } => y + z.0,
    //     Struct { x: false, y, z } => y + z.0,
    //     Struct { .. } => 0,
    //     TODO
    // };
    
    let t1 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t1 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        _ => 0,
        (false, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        _ => 0,
        _ => 0,
    };

    let t2 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t2 {
        (true, _, s, n) => n + s.y,
        x => x.3,
        (false, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        x => x.3,
        x => x.3,
        _ => 0,
    };

    let t3 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t3 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (b, e, s, n) => {
            poke(e);
            if b { s.y } else { n }
        },
        (false, _, s, n) => n + s.y,
        (b, e, s, n) => {
            poke(e);
            if b { s.y } else { n }
        },
        (b, e, s, n) => {
            poke(e);
            if b { s.y } else { n }
        },
        _ => 0,
    };

    let t4 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t4 {
        (_, _, _, _) => 0,
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (_, _, _, _) => 0,
        (_, _, _, _) => 0,
        _ => 0,
    };

    let t5 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t5 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (_, _, _, n) => n,
        (false, _, s, n) => n + s.y,
        (_, _, _, n) => n,
        (_, _, _, n) => n,
        _ => 0,
    };

    let t6 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t6 {
        (_, _, Struct { x, y, z: (j , k, l)}, n) => {
            poke(x);
            poke(y);
            poke(j);
            poke(k);
            poke(l);
            n
        },
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
        (_, _, Struct { x, y, z: (j , k, l)}, n) => {
            poke(x);
            poke(y);
            poke(j);
            poke(k);
            poke(l);
            n
        },
        _ => 0,
    };

    let t7 = (false, Enum::A, Struct::new(), 0u64);

    let _x = match t7 {
        (true, _, s, n) => n + s.y,
        (false, _, s, n) => n + s.y,
        (_, _, Struct { x: _, y: _, z: (_ , k, _)}, n) => {
            poke(k);
            n
        },
        (false, _, s, n) => n + s.y,
        (_, _, Struct { x: _, y: _, z: (_ , k, _)}, n) => {
            poke(k);
            n
        },
        (_, _, Struct { x: _, y: _, z: (_ , k, _)}, n) => {
            poke(k);
            n
        },
        _ => 0,
    };

    let or_no_warning = 0;

    let _x = match or_no_warning {
        1 | 2 | 3 => 0,
        4 => 0,
        5 => 0,
        _ => 0,
    };

    let or1 = 0u64;

    let _x = match or1 {
        1 | 2 | _ => 0,
        4 => 0,
        5 => 0,
        _ => 0,
    };

    let or2 = 0u64;

    let _x = match or2 {
        4 => 0,
        1 | _ | 2 => 0,
        5 => 0,
        _ => 0,
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
    
    // TODO: Once internal compiler error is solved (https://github.com/FuelLabs/sway/issues/5106) add examples similar to this case: 
    // let e = EnumB::A;
 
    // let _x = match e {
    //     EnumB::A | EnumB::B | _ => 0,
    //     EnumB::A => 0,
    //     EnumB::B => 0,
    // };

    poke(Enum::B);
    poke(Enum::C);
    poke(Enum::D);
    poke(Enum::E(0));
    poke(Struct::new().use_me());
}

fn poke<T>(_x: T) { }
