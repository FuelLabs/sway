contract;

abi ConstantId {
    const ID: u32 = 0;
}

impl ConstantId for Contract {
    const ID: u32 = 1;
}

fn main() -> u32 {
  let _ = ConstantId::ID;

  // Leave enough space to avoid having both `let` lines in both error messages.

  let _ = Contract::ID;
}
