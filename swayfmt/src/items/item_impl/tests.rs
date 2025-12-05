use forc_diagnostic::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_item;

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
        None::<T>(())
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
                    None::<T>(())
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

fmt_test_item!(    impl_empty_fn_args
"impl TestContract for Contract {
    fn return_configurables() -> (u8, bool, [u32; 3], str[4], StructWithGeneric<u8>, EnumWithGeneric<bool>) {
        (U8, BOOL, ARRAY, STR_4, STRUCT, ENUM)
    }
}",
            intermediate_whitespace
            "impl TestContract for Contract {
    fn return_configurables(    ) -> ( u8, bool, [u32; 3], str[4], StructWithGeneric<u8>, EnumWithGeneric<bool> 
    ) {
        ( U8, BOOL,  ARRAY, STR_4 , STRUCT, ENUM )
    }
}
"
);

fmt_test_item!(    impl_empty_fn_comment
"impl MyAbi for Contract {
    fn foo() {
        // ... logic ...
    }
}",
            intermediate_whitespace
"impl   MyAbi for Contract {
fn foo(  ) {
            // ... logic ...
}
}"
);

fmt_test_item!(impl_contains_const
"impl ConstantId for Struct {
    const ID: u32 = 5;
}",
intermediate_whitespace
"impl ConstantId for Struct {
    const ID: u32=5;
}"
);

fmt_test_item!(impl_for_struct_where_clause
"impl MyStructWithWhere<T, A>
where
    T: Something,
    A: Something,
{
    fn do() {}
}",
intermediate_whitespace
"impl MyStructWithWhere<T, A> where T: Something, A: Something { fn do() {} }"
);

fmt_test_item!(impl_trait_for_struct_where_clause
"impl MyTrait<T, A> for MyStructWithWhere<T, A>
where
    T: Something,
    A: Something,
{
    fn do() {}
}",
intermediate_whitespace
"impl MyTrait<T, A> for MyStructWithWhere<T, A> where T: Something, A: Something { fn do() {} }"
);
