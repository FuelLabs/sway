script;

struct TestStruct {
    val: u64,
}

fn main(baba: TestStruct, keke: TestStruct) -> u64 {
    baba.val + keke.val
}
