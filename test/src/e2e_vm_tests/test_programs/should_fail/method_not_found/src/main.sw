contract;

struct ReturnedStruct {}

trait TraitA {
  fn associated() -> ReturnedStruct;
  fn method(self);
  fn associated_with_params(x: u64);
  fn method_with_params(self, x: u64);
  fn different_method(self);
  fn different_associated();
}

trait TraitB {
  fn method_b(self);
  fn associated_b();
}

trait TraitBB {
  fn method_b(self, x: u64, y: u64);
}

struct Struct {}

impl Struct {
  fn associated() -> ReturnedStruct { ReturnedStruct { } }
  fn method(self) {}
  fn associated_with_params(_x: u64) {}
  fn method_with_params(self, _x: u64) {}
  fn different_method(self, _x: u64) {}
  fn different_associated(_x: u64) {}
}

impl TraitA for Struct {
  fn associated() -> ReturnedStruct { ReturnedStruct { } }
  fn method(self) {}
  fn associated_with_params(_x: u64) {}
  fn method_with_params(self, _x: u64) {}
  fn different_method(self) {}
  fn different_associated() {}
}

impl TraitB for Struct {
  fn method_b(self) {}
  fn associated_b() {}
}

impl TraitBB for Struct {
  fn method_b(self, _x: u64, _y: u64) {}
}

struct GenericStruct<T> {}

impl GenericStruct<bool> {
  fn associated_on_generic_struct(self) {}
  fn method_on_generic_struct(self) {}
}

impl GenericStruct<u8> {
  fn associated_on_generic_struct(self) {}
  fn method_on_generic_struct(self) {}
}

fn generic<T>(t: T) where T: TraitB {
  t.method();
  t.method_b(42);
  T::associated();
  T::associated_b(42);
}

trait ContractTrait {
    fn fn_in_contract_trait(x: u64);
}

abi Abi: ContractTrait {
} {
    fn provided_fn_in_abi(x: u64) {
        Self::fn_in_contract_trait(x);
    }
}

impl ContractTrait for Contract {
    fn fn_in_contract_trait(_x: u64) {}
}

impl Abi for Contract {}

impl Contract {
  fn main() {
    let s = Struct {};
    let _: ReturnedStruct = s.associated();

    let _: ReturnedStruct = Struct::method();

    let _: ReturnedStruct =s.method(42);
    let _: ReturnedStruct =Struct::associated(42);

    s.different_method(true);
    Struct::different_associated(true);

    Struct::associated_with_params();

    s.method_with_params();

    s.method_b(42);

    let gs_u64 = GenericStruct::<u64> {};
    gs_u64.method_on_generic_struct();

    GenericStruct::<u64>::associated_on_generic_struct();

    generic(s);

    let caller = abi(Abi, 0x0000000000000000000000000000000000000000000000000000000000000000);
    caller.provided_fn_in_abi();
    caller.provided_fn_in_abi(true);

    let _ = 0u8 + true;
  }
}