contract;

#[abi_name(name = "RenamedMyStruct")]
struct MyStruct {}

#[abi_name(name = "RenamedMyEnum")]
enum MyEnum {
  A: ()
}

abi MyAbi {
    fn my_struct() -> MyStruct;
    fn my_enum() -> MyEnum;
}

impl MyAbi for Contract {
  fn my_struct() -> MyStruct { MyStruct{} }
  fn my_enum() -> MyEnum { MyEnum::A }
}
