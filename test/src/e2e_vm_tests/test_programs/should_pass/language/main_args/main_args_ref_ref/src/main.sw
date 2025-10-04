script;

struct TestStruct {
    val: u64,
}

fn main(baba: TestStruct, keke: TestStruct) -> (TestStruct, TestStruct) {
    (
        TestStruct {
            val: baba.val + keke.val
        },
        TestStruct {
            val: baba.val + keke.val
        }
    )    
}
