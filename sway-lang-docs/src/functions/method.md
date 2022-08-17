# Methods

Methods are similar to functions in that we declare them with the `fn` keyword and they have parameters and return a value. However, unlike functions, _Methods_ are defined within the context of a struct (or enum), and either refers to that type or mutates it. The first parameter of a method is always `self`, which represents the instance of the struct the method is being called on.

To declare methods and associated functions for a struct or enum, use an _impl block_. Here, `impl` stands for implementation.

```sway
script;

struct Foo {
    bar: u64,
    baz: bool,
}

impl Foo {
    // this is a _method_, as it takes `self` as a parameter.
    fn is_baz_true(self) -> bool {
        self.baz
    }
}

fn main() {
    let foo = Foo { bar: 42, baz: true };
    assert(foo.is_baz_true());
}
```

To call a method, simply use dot syntax: `foo.iz_baz_true()`.
