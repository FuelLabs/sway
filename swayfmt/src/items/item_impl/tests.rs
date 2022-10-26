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
                let formatted_code = crate::parse::parse_format::<sway_ast::ItemImpl>($y);
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

fmt_test!(  impl_with_nested_items
"impl AuthTesting for Contract {
    fn returns_msg_sender(expected_id: ContractId) -> bool {
        let result: Result<Identity, AuthError> = msg_sender();
        let mut ret = false;
        if result.is_err() {
            ret = false;
        }
        let unwrapped = result.unwrap();
        match unwrapped {
            Identity::ContractId(v) => {
                ret = true
            }
            _ => {
                ret = false
            }
        }
        ret
    }
}",
            intermediate_whitespace
"impl AuthTesting for Contract {
    fn returns_msg_sender(expected_id: ContractId) -> bool {
        let result: Result<Identity, AuthError> = msg_sender();
        let mut ret = false;
        if result.is_err() {
            ret = false;
        }
        let unwrapped = result.unwrap();
        match unwrapped {
            Identity::ContractId(v) => {ret = true}
            _ => {ret = false}
        }
        ret
    }
}"
);

fmt_test!(  normal_with_generics
"impl<T> Option<T> {
    fn some(value: T) -> Self {
        Option::Some::<T>(value)
    }
    fn none() -> Self {
        Option::None::<T>(())
    }
    fn to_result(self) -> Result<T> {
        if let Option::Some(value) = self {
            ~Result::<T>::ok(value)
        } else {
            ~Result::<T>::err(99u8)
        }
    }
}",
            intermediate_whitespace
            "impl<T> Option<T> {
                fn some(value: T) -> Self {
                    Option::Some::<T>(value)
                }
                fn none() -> Self {
                    Option::None::<T>(())
                }
                fn to_result(self) -> Result<T> {
                    if let Option::Some(value) = self {
                    ~Result::<T>::ok(value)
                    } else {
                    ~Result::<T>::err(99u8)
                    }
                }
            }"
);
