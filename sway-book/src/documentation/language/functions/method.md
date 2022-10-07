# Methods

Methods are defined within the context of a [struct](../built-ins/structs.md) (or [enum](../built-ins/enums.md)) and either refer to the type or mutate it. The first parameter of a method is always `self`, which represents the instance of the struct the method is being called on.

### Decleration

To declare a method for a struct or enum, use an _impl block_. Here, `impl` stands for implementation.

```sway
struct Foo {
    bar: u64,
    baz: bool,
}

impl Foo {
    // this is a method because it takes `self` as a parameter
    fn is_baz_true(self) -> bool {
        self.baz
    }

    // methods can take any number of parameters
    fn add_number(self, number: u64) -> u64 {
        self.bar + number
    }
}
```

### Usage

To call a method use the dot syntax: `<variable name>.<method name>()`.

```sway
fn main() {
    let foo = Foo { bar: 42, baz: true };
    let result = foo.is_baz_true();  // evaluates to `true`
    let result = foo.add_number(5);  // evaluates to `47`
}
```
