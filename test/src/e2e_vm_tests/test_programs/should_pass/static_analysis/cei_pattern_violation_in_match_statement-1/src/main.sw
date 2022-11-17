contract;

use std::storage::store;

enum MyEnum {
  A: (),
  B: (),
  C: (),
}

abi TestAbi {
  #[storage(write)]
  fn deposit(e: MyEnum);
}

impl TestAbi for Contract {
  #[storage(write)]
  fn deposit(e: MyEnum) {
    // interaction in the matchee, effect in a branch
    match
      {
        // interaction
        abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit(e);
        e
      } {
        MyEnum::A => {
          match e {
            MyEnum::A => {
            },
            MyEnum::B => {
            },
            MyEnum::C => {
              // effect -- therefore violation of CEI where effect should go before interaction
              {
                store(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, ())
              }
            },
          }
        },
        MyEnum::B => {
        },
        MyEnum::C => {
        },
      }
  }
}
