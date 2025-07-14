contract;

#[abi_name(name = "SameName")]
struct MyStruct {}

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

// OK because enums are on a different namespace
#[abi_name(name = "SameName")]
enum MyEnum {
  A: ()
}

abi MyAbi {
    fn my_struct() -> MyStruct;
    fn my_struct1() -> MyStruct1;
    fn my_struct2() -> MyStruct2;
    fn my_struct3() -> MyStruct3;
    fn my_struct4() -> MyStruct4;
    fn my_struct5() -> MyStruct5;
    fn my_enum() -> MyEnum;
}

impl MyAbi for Contract {
  fn my_struct() -> MyStruct { MyStruct{} }
  fn my_struct1() -> MyStruct1 { MyStruct1{} }
  fn my_struct2() -> MyStruct2 { MyStruct2{} }
  fn my_struct3() -> MyStruct3 { MyStruct3{} }
  fn my_struct4() -> MyStruct4 { MyStruct4{} }
  fn my_struct5() -> MyStruct5 { MyStruct5{} }
  fn my_enum() -> MyEnum { MyEnum::A }
}
