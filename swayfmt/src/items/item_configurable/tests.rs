use forc_tracing::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};

macro_rules! fmt_test {
    ($scope:ident $desired_output:expr, $($name:ident $y:expr),+) => {
        fmt_test_inner!($scope $desired_output,
                                $($name $y)+
                                ,
                                remove_trailing_whitespace format!("{} \n\n\t ", $desired_output).as_str(),
                                remove_beginning_whitespace format!("  \n\t{}", $desired_output).as_str(),
                                identity $desired_output, /* test return is valid */
                                remove_beginning_and_trailing_whitespace format!("  \n\t  {} \n\t   ", $desired_output).as_str()
                       );
    };
}

macro_rules! fmt_test_inner {
    ($scope:ident $desired_output:expr, $($name:ident $y:expr),+) => {
        $(
        paste! {
            #[test]
            fn [<$scope _ $name>] () {
                let formatted_code = crate::parse::parse_format::<sway_ast::ItemConfigurable>($y);
                let changeset = diff_lines(&formatted_code, $desired_output);
                let diff = changeset.diff();
                let count_of_updates = diff.len();
                if count_of_updates != 0 {
                    println!("FAILED: {count_of_updates} diff items.");
                }
                for diff in diff {
                    match diff {
                        DiffOp::Equal(old) => {
                            for o in old {
                                println!("{}", o)
                            }
                        }
                        DiffOp::Insert(new) => {
                            for n in new {
                                println_green(&format!("+{}", n));
                            }
                        }
                        DiffOp::Remove(old) => {
                            for o in old {
                                println_red(&format!("-{}", o));
                            }
                        }
                        DiffOp::Replace(old, new) => {
                            for o in old {
                                println_red(&format!("-{}", o));
                            }
                            for n in new {
                                println_green(&format!("+{}", n));
                            }
                        }
                    }
                }
                assert_eq!(&formatted_code, $desired_output)
            }
        }
    )+
}
}

fmt_test!( configurables
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
