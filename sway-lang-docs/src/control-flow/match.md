# match

Sway supports advanced pattern matching through exhaustive `match` expressions.

```sway
enum Foo {
    One: (),
    Two: (),
    Three: (),
}

fn main() {
    let one = Foo::One;
    let two = Foo::Two;
    let three = Foo::Three;

    let mut result = 0;
    
    if let Foo::One = one {
        result = 1;
    }
}
```

```sway
{{#include ../../../examples/match_statements/src/main.sw}}
```

In the example above, braces around the code block following `=>` in each match arm are not required unless the code block contains multiple statements. They are added in this example due to an [issue in the Sway formatter](https://github.com/FuelLabs/sway/issues/604).
