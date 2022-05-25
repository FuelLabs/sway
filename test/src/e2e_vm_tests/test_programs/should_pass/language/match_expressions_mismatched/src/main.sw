script;

struct MyStruct { a: u64, b: u64 }

enum MyEnum {
  Variant1: (),
  Variant2: u64,
  Variant3: MyStruct,
}

fn main() -> u64 {
  let x = MyEnum::Variant1;
  let y = MyEnum::Variant2 ( 5 ) ;
  let z = MyEnum::Variant3 ( MyStruct { a: 0, b: 1 } ) ;

  match y {
    MyEnum::Variant2 ( y ) => y,
    _ => 10,
  }
}