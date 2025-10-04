script;

struct TestStruct {
    val: u64,
}

fn main(baba: TestStruct) -> TestStruct {
    TestStruct {
        val: baba.val + 1
    }    
}
