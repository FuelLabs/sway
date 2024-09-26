// This tests proves that https://github.com/FuelLabs/sway/issues/6379 is fixed.

script;

trait Build {
    fn build() -> Self;
}

fn produce<T>() -> T where T: Build {
    T::build()
}

impl Build for u8 {
    fn build() -> Self {
        7
    }
}

impl Build for u32 {
    fn build() -> Self {
        31
    }
}

impl Build for u64 {
    fn build() -> Self {
        63
    }
}

fn consume_u8(a: u8) -> u8 {
    a
}

fn consume_u32(a: u32) -> u32 {
    a
}

fn consume_u64(a: u64) -> u64 {
    a
}

fn main() -> (u8, u32, u64) {
    let a = produce_consume_u8();
    let b = produce_consume_u32();
    let c = produce_consume_u64();

    (a, b, c)
}

fn produce_consume_u8() -> u8 {
    let x = produce();
    consume_u8(x)
}

fn produce_consume_u32() -> u32 {
    let x = produce();
    consume_u32(x)
}

fn produce_consume_u64() -> u64 {
    let x = produce();
    consume_u64(x)
}