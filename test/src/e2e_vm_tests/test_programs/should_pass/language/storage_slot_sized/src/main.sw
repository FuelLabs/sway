contract;

struct S1 {
  f1: u64,
  f2: u64,
  f3: u64,
  f4: u64,
}

impl PartialEq for S1 {
  fn eq(self, other: Self) -> bool {
      self.f1 == other.f1 && self.f2 == other.f2 && self.f3 == other.f3 && self.f4 == other.f4
  }
}

impl Eq for S1 {}

storage {
    var1: S1 = S1 { f1: 1, f2: 2, f3: 3, f4: 4 },
    var2: (S1, S1) = (S1 { f1: 11, f2: 21, f3: 31, f4: 41 }, S1 { f1: 12, f2: 22, f3: 32, f4: 42 })
}

abi StorageTest {
    #[storage(write)]
    fn store_something_1(x: S1);

    #[storage(read)]
    fn check_store_1(x: S1);

    #[storage(write)]
    fn store_something_2(x: (S1, S1));

    #[storage(read)]
    fn check_store_2(x: (S1, S1));
}

impl StorageTest for Contract {
    #[storage(write)]
    #[inline(never)]
    fn store_something_1(x: S1) {
        storage.var1.write(x);
    }

    #[storage(read)]
    #[inline(never)]
    fn check_store_1(x: S1) {
       assert(storage.var1.read() == x);
    }

    #[storage(write)]
    #[inline(never)]
    fn store_something_2(x: (S1, S1)) {
        storage.var2.write(x);
    }

    #[storage(read)]
    #[inline(never)]
    fn check_store_2(x: (S1, S1)) {
       assert(storage.var2.read() == x);
    }
}

#[test]
fn test_store_something() {
    let storage_test = abi(StorageTest, CONTRACT_ID);
    let x1 = S1 { f1: 1, f2: 2, f3: 3, f4: 4 };
    storage_test.check_store_1(x1);
    let x2 = S1 { f1: 2, f2: 3, f3: 4, f4: 5 };
    storage_test.store_something_1(x2);
    storage_test.check_store_1(x2);

    let x3 = (S1 { f1: 11, f2: 21, f3: 31, f4: 41 }, S1 { f1: 12, f2: 22, f3: 32, f4: 42 });
    let x4 = (S1 { f1: 111, f2: 211, f3: 311, f4: 411 }, S1 { f1: 121, f2: 221, f3: 321, f4: 421 });
    storage_test.check_store_2(x3);
    storage_test.store_something_2(x4);
    storage_test.check_store_2(x4)
}
