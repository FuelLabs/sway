library;

// ANCHOR: definition
struct Foo {
    pub bar: u64,
    baz: bool,
}
// ANCHOR_END: definition
// ANCHOR: instantiation
fn hardcoded_instantiation() {
    // Instantiate the variable `foo` as `Foo`
    let mut foo = Foo {
        bar: 42,
        baz: false,
    };

    // Access and write to "baz"
    foo.baz = true;
}

fn variable_instantiation() {
    // Declare variables and pass them into `Foo`
    let number = 42;
    let boolean = false;

    let mut foo = Foo {
        bar: number,
        baz: boolean,
    };

    // Access and write to "baz"
    foo.baz = true;
}

fn shorthand_instantiation() {
    // Declare variables with the same names as the fields in `Foo`
    let bar = 42;
    let baz = false;

    // Instantiate `foo` as `Foo`
    let mut foo = Foo { bar, baz };

    // Access and write to "baz"
    foo.baz = true;
}
// ANCHOR_END: instantiation
// ANCHOR: destructuring
fn destructuring() {
    let foo = Foo {
        bar: 42,
        baz: false,
    };

    // bar and baz are now accessible as variables
    let Foo { bar, baz } = foo;

    if baz {
        let quix = bar * 2;
    }

    // You may use `..` to omit the remaining fields if the types match
    // The compiler will fill them in for you
    let Foo { bar, .. } = foo;
}
// ANCHOR_END: destructuring
