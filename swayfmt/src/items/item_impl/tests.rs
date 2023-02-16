use forc_tracing::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::{fmt_test, fmt_test_inner, fmt_test_item};

fmt_test_item!(  impl_with_nested_items
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

fmt_test_item!(  normal_with_generics
"impl<T> Option<T> {
    fn some(value: T) -> Self {
        Option::Some::<T>(value)
    }
    fn none() -> Self {
        Option::None::<T>(())
    }
    fn to_result(self) -> Result<T> {
        if let Option::Some(value) = self {
            Result::<T>::ok(value)
        } else {
            Result::<T>::err(99u8)
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
                    Result::<T>::ok(value)
                    } else {
                    Result::<T>::err(99u8)
                    }
                }
            }"
);
