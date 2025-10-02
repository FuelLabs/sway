script;

struct TestStruct {
    val: u64,
}

fn main(baba: TestStruct, keke: u64) -> (TestStruct, u64) {
    (
        TestStruct {
            val: baba.val + keke
        },
        baba.val + keke
    )
}
