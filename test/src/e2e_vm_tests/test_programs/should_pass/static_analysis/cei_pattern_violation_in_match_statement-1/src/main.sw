contract;

use std::storage::storage_api::{write_quads, write_slot};

enum MyEnum {
  A: (),
  B: (),
  C: (),
}

abi TestAbi {
  #[storage(write)]
  fn deposit_quads(e: MyEnum);
  #[storage(write)]
  fn deposit_slot(e: MyEnum);
}

impl TestAbi for Contract {
  #[storage(write)]
  fn deposit_quads(e: MyEnum) {
    // interaction in the matchee, effect in a branch
    match
      {
        // interaction
        abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit_quads(e);
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
                write_quads(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, 0, ());
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

  #[storage(write)]
  fn deposit_slot(e: MyEnum) {
    // interaction in the matchee, effect in a branch
    match
      {
        // interaction
        abi(TestAbi, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae).deposit_slot(e);
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
                write_slot(0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae, ());
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
