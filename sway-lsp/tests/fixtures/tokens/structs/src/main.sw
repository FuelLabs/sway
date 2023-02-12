contract;

/// My data enum
enum Data {
    First: (),
    Second: (),
}

/// My struct type
struct MyStruct<T, U> {
    g: U,
    x: T,
    y: Data,
    z: (u64, Data),
    t: [Data; 5],
    j: (u32, (Data, [Data; 2])),
}

struct Simple {
    x: u8,
}

fn func() {
    let x = Simple { 
        x: 7
    };
}