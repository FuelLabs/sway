script;

struct Pair {
    x: u64,
    y: u64,
}

struct MyCoolStruct {
    tuple0: (u64, u32),
    tuple1: (u32, u64),
    pair: Pair,
    tuple_of_pairs: (Pair, Pair),
}

fn main() -> u64 {
    let mut foo0: (u32, u64) = (1, 2);
    foo0.0 = 3;
    foo0.1 = 4;

    let mut foo1: (Pair, u64) = (Pair { x: 5, y: 6 }, 7);
    foo1.0.x = 8;
    foo1.0.y = 9;
    foo1.1 = 10;

    let mut foo2: (u32, Pair) = (11, Pair { x: 12, y: 13 });
    foo2.0 = 14;
    foo2.1.x = 15;
    foo2.1.y = 16;

    let mut foo3: MyCoolStruct = MyCoolStruct {
        tuple0: (17, 18),
        tuple1: (19, 20),
        pair: Pair { x: 21, y: 22 },
        tuple_of_pairs: (Pair { x: 23, y: 24 }, Pair { x: 25, y: 26 }),
    };
    foo3.tuple0.0 = 27;
    foo3.tuple0.1 = 28;
    foo3.tuple1.0 = 29;
    foo3.tuple1.1 = 30;
    foo3.pair.x = 31;
    foo3.pair.y = 32;
    foo3.tuple_of_pairs.0.x = 33;
    foo3.tuple_of_pairs.0.y = 34;
    foo3.tuple_of_pairs.1.x = 35;
    foo3.tuple_of_pairs.1.y = 36;

    let ret = {
        0
        + foo0.1
        + foo1.0.x
        + foo1.0.y
        + foo1.1
        + foo2.1.x
        + foo2.1.y
        + foo3.tuple0.0
        + foo3.tuple1.1
        + foo3.pair.x
        + foo3.pair.y
        + foo3.tuple_of_pairs.0.x
        + foo3.tuple_of_pairs.0.y
        + foo3.tuple_of_pairs.1.x
        + foo3.tuple_of_pairs.1.y
    };

    // 320
    return ret;
}

