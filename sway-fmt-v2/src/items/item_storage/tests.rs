use crate::{Format, Formatter};
use forc_util::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use sway_ast::ItemStorage;
use sway_parse::{handler::Handler, *};

fn format_code(input: &str) -> String {
    let mut formatter: Formatter = Default::default();
    let input_arc = std::sync::Arc::from(input);
    let token_stream = lex(&input_arc, 0, input.len(), None).unwrap();
    let handler = Handler::default();
    let mut parser = Parser::new(&token_stream, &handler);
    let expression: ItemStorage = parser.parse().unwrap();

    let mut buf = Default::default();
    expression.format(&mut buf, &mut formatter).unwrap();

    buf
}

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
                let formatted_code = format_code($y);
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

fmt_test!(  storage_maps
"storage {
    map1: StorageMap<u64, bool> = StorageMap {},
    map2: StorageMap<u64, u8> = StorageMap {},
    map3: StorageMap<u64, u16> = StorageMap {},
    map4: StorageMap<u64, u32> = StorageMap {},
    map5: StorageMap<u64, u64> = StorageMap {},
    map6: StorageMap<u64, (b256, u8, bool)> = StorageMap {},
    map7: StorageMap<u64, Struct> = StorageMap {},
    map8: StorageMap<u64, Enum> = StorageMap {},
    map9: StorageMap<u64, str[33]> = StorageMap {},
    map10: StorageMap<u64, [b256; 3]> = StorageMap {},
    map11: StorageMap<bool, u64> = StorageMap {},
    map12: StorageMap<u8, u64> = StorageMap {},
    map13: StorageMap<u16, u64> = StorageMap {},
    map14: StorageMap<u32, u64> = StorageMap {},
    map15: StorageMap<(b256, u8, bool), u64> = StorageMap {},
    map16: StorageMap<Struct, u64> = StorageMap {},
    map17: StorageMap<Enum, u64> = StorageMap {},
    map18: StorageMap<str[33], u64> = StorageMap {},
    map19: StorageMap<[b256; 3], u64> = StorageMap {},
}",
            wrong_new_lines
"storage {
    map1: StorageMap<u64,
    bool> = StorageMap {
    },
    map2: StorageMap<u64,
    u8> = StorageMap {
    },
    map3: StorageMap<u64,
    u16> = StorageMap {
    },
    map4: StorageMap<u64,
    u32> = StorageMap {
    },
    map5: StorageMap<u64,
    u64> = StorageMap {
    },
    map6: StorageMap<u64,
    (b256, u8, bool) > = StorageMap {
    },
    map7: StorageMap<u64,
    Struct> = StorageMap {
    },
    map8: StorageMap<u64,
    Enum> = StorageMap {
    },
    map9: StorageMap<u64,
    str[33]> = StorageMap {
    },
    map10: StorageMap<u64,
    [b256;
    3]> = StorageMap {
    },
    map11: StorageMap<bool,
    u64> = StorageMap {
    },
    map12: StorageMap<u8,
    u64> = StorageMap {
    },
    map13: StorageMap<u16,
    u64> = StorageMap {
    },
    map14: StorageMap<u32,
    u64> = StorageMap {
    },
    map15: StorageMap<(b256,
    u8, bool), u64 > = StorageMap {
    },
    map16: StorageMap<Struct,
    u64> = StorageMap {
    },
    map17: StorageMap<Enum,
    u64> = StorageMap {
    },
    map18: StorageMap<str[33],
    u64> = StorageMap {
    },
    map19: StorageMap<[b256;
    3],
    u64> = StorageMap {
    },
}"
);
