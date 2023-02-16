contract;

struct TestStruct<A, B> {}

trait TestTrait<A, B> {}

impl<A, B> TestStruct<A, B> {}
impl<A, B> TestTrait<A, B> for TestStruct<A, B> {}
impl<A, B> TestStruct<TestStruct<A, B>, B> {}
