contract;

mod other;
use other::OtherEnum;
use other::OtherStruct;

#[abi_name(name = "SameName")]
struct MyStruct {}

#[abi_name(name = "MyStruct")]
struct MyStruct0 {}

#[abi_name(name = "SameName")]
struct MyStruct1 {}

#[abi_name(name = "SameName")]
struct MyStruct2 {}

#[abi_name(name = "")]
struct MyStruct3 {}

#[abi_name(name = "this !s n0t an identif1er")]
struct MyStruct4 {}

#[abi_name(name = "this::looks::like::a::path")]
struct MyStruct5 {}

#[abi_name(name = "OtherStruct")]
struct MyStruct6 {}

#[abi_name(name = "::some_module::in_the_same::package")]
struct MyStruct7 {}

// OK because enums are on a different namespace
#[abi_name(name = "SameName")]
enum MyEnum {
  A: ()
}

#[abi_name(name = "OtherEnum")]
enum MyEnum1 {
  A: ()
}

abi MyAbi {
    fn other_struct() -> OtherStruct;
    fn my_struct() -> MyStruct;
    fn my_struct0() -> MyStruct0;
    fn my_struct1() -> MyStruct1;
    fn my_struct2() -> MyStruct2;
    fn my_struct3() -> MyStruct3;
    fn my_struct4() -> MyStruct4;
    fn my_struct5() -> MyStruct5;
    fn my_struct6() -> MyStruct6;
    fn my_struct7() -> MyStruct7;
    fn other_enum() -> OtherEnum;
    fn my_enum() -> MyEnum;
    fn my_enum1() -> MyEnum1;
}

impl MyAbi for Contract {
  fn other_struct() -> OtherStruct { OtherStruct{} }
  fn my_struct() -> MyStruct { MyStruct{} }
  fn my_struct0() -> MyStruct0 { MyStruct0{} }
  fn my_struct1() -> MyStruct1 { MyStruct1{} }
  fn my_struct2() -> MyStruct2 { MyStruct2{} }
  fn my_struct3() -> MyStruct3 { MyStruct3{} }
  fn my_struct4() -> MyStruct4 { MyStruct4{} }
  fn my_struct5() -> MyStruct5 { MyStruct5{} }
  fn my_struct6() -> MyStruct6 { MyStruct6{} }
  fn my_struct7() -> MyStruct7 { MyStruct7{} }
  fn other_enum() -> OtherEnum { OtherEnum::A }
  fn my_enum() -> MyEnum { MyEnum::A }
  fn my_enum1() -> MyEnum1 { MyEnum1::A }
}
