# Libraries

```sway
// note that libraries must be named, so we know how to refer to them and import things.
library my_library;

// All public items in a library are made available to other projects which import this library.
pub struct MyStruct {
    field_one: u64,
    field_two: bool,
}
```
