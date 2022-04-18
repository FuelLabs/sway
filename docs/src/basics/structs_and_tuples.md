# Structs and Tuples

## Structs

Structs in Sway are a named grouping of types. You may also be familiar with structs via another name: _product types_. Sway does not make any significantly unique usages of structs; they are similar to most other languages which have structs. If you're coming from an object-oriented background, a struct is like the data attributes of an object.

To declare a struct type, use _struct declaration syntax_:

```sway
struct Foo {
    bar: u64,
    baz: bool,
}
```

This is saying that we have some structure named `Foo`. `Foo` has two fields: `bar`, a `u64`; and `baz`, a `bool`. To instantiate the structure `Foo`, we can use _struct instantiation syntax_, which is very similar to the declaration syntax except with expressions in place of types.

```sway
let foo = Foo {
    bar: 42,
    baz: false,
};
```

To access a field of a struct, use _struct field access syntax_:

```sway
// Instantiate `foo`.
let foo = Foo {
    bar: 42,
    baz: true,
};

// Access field `baz` of `foo`.
assert(foo.baz);
```

### Struct Memory Layout

_This information is not vital if you are new to the language, or programming in general._

Structs have zero memory overhead. What that means is that in memory, each struct field is laid out sequentially. No metadata regarding the struct's name or other properties is preserved at runtime. In other words, structs are compile-time constructs. This is the same in Rust, but different in other languages with runtimes like Java.

## Tuples

Tuples are a [basic static-length type](./built_in_types.md#tuple-types) which contain multiple different types within themselves. The type of a tuple is defined by the types of the values within it, and a tuple can contain basic types as well as structs and enums. An example of a tuple declaration is provided below.

```sway
let my_tuple: (u64, bool, u64) = (100, false, 10000);
```

The values within this tuple can then be accessed with a `.` syntax in order of the type, starting at an index of 0 like shown in the example provided below.

```sway
let x: u64 = my_tuple.0;
let y: bool = my_tuple.1;
```

Tuples can also contain tuples within themselves, and be used in destructing syntax to declare multiple values at once.

Common usecases for tuples are returning multiple values from a function, packing parameters into a function, or storing a series of related values.
## Enums

_Enumerations_, or _enums_, are also known as _sum types_. An enum is a type that could be one of several variants. To declare an enum, you enumerate all potential variants. Let's look at _enum declaration syntax_:

```sway
enum Color {
    Blue: (),
    Green: (),
    Red: (),
    Silver: (),
    Grey: (),
}
```

Here, we have defined five potential colors. Each enum variant is just the color name. As there is no extra data associated with each variant, we say that each variant is of type `()`, or unit. It is also possible to have an enum variant contain extra data. Take a look at this more substantial example, which combines struct declarations with enum variants:

```sway
use std::collections::Vec;
use inventory_system::InventoryItem;
use inventory_system::Insurer;

struct Claim {
    insurance_company: Insurer,
    item_number: u64,
    item_cost: u64,
}

struct Receipt {
    customer: CustomerId,
    items_purchased: Vec<InventoryItem>,
}

struct Refund {
    customer: CustomerId,
    items_returned: Vec<InventoryItem>,
}

enum InventoryEvent {
    CustomerPurchase : Receipt,
    ItemLoss         : Claim,
    CustomerReturn   : Refund,
}
```

```sway
enum Color {
    Blue: (),
    Green: (),
    Red: (),
    Silver: (),
    Grey: (),
}

fn main() {
    let color = Color::Blue;
}
```

Here, we have instantiated a variable named `color` with _enum instantiation syntax_. Note that enum instantiation does not require the `~` tilde syntax. If we wanted to instantiate an enum with some interior data, it looks like this:

```sway
struct Claim {
    insurance_company: Insurer,
    item_number: u64,
    item_cost: u64,
}

let event = InventoryEvent::ItemLoss(Claim {
    insurance_company: ~Insurer::default(),
    item_number: 42,
    item_cost: 1_000,
});
```

### Enum Memory Layout

_This information is not vital if you are new to the language, or programming in general._

Enums do have some memory overhead. To know which variant is being represented, Sway stores a one-word (8-byte) tag for the enum variant. The space reserved after the tag is equivalent to the size of the _largest_ enum variant. So, to calculate the size of an enum in memory, add 8 bytes to the size of the largest variant. For example, in the case of `Color` above, where the variants are all `()`, the size would be 8 bytes since the size of the largest variant is 0 bytes.
