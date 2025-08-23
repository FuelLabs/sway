use forc_tracing::{println_green, println_red};
use pastey::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_item;

fmt_test_item!( configurables
"configurable {
    C0: bool = true,
    C1: u64 = 42,
    C2: b256 = 0x1111111111111111111111111111111111111111111111111111111111111111,
    C3: MyStruct = MyStruct { x: 42, y: true },
    C4: MyEnum = MyEnum::A(42),
    C5: MyEnum = MyEnum::B(true),
    C6: str[4] = \"fuel\",
    C7: [u64; 4] = [1, 2, 3, 4],
    C8: u64 = 0,
}",
            wrong_new_lines
"configurable {
    C0: bool = true, C1: u64 = 42,
        C2:     b256   = 

        0x1111111111111111111111111111111111111111111111111111111111111111,
    C3: MyStruct = 
    MyStruct { x: 42, 
    y: true },
    C4: MyEnum 
    = MyEnum::A(42),
    C5: MyEnum = MyEnum::B(true),
    C6: str[4] = \"fuel\",
    C7: [u64; 4] = [1, 2, 
    3, 4], C8: u64 = 0,
}"
);
