script;

#[inline(never)]
fn f(baba:u64) -> u64 {
    if baba == 0 {
        1
    } else {
        2
    }
}

fn main(baba: u64) -> u64 {
    f(baba)
}
