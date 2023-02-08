contract;

dep traits;

use traits::{Test1, Test2};

trait A: Test1 {}
trait B: Test1 + Test2 {}

struct S {}
impl Test1 for S {}