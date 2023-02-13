contract;

/// Test Struct Docs
struct TestStruct {
    foo: u8,
}

/// Color enum with RGB variants
enum Color {
    Red: (),
    Green: (),
    Blue: (),
}

/// My Enum documentation
pub enum MyEnum {
    First: TestStruct,
    Second: Color,
    Third: (u8, Color),
    /// Docs for variants
    Fourth: u8,
}

fn func() {
    let x = Color::Red;
    let y: MyEnum = MyEnum::Fourth(8);
}
