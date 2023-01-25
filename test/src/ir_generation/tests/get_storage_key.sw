contract;

struct Empty {}

impl Empty {
    fn bar(self) -> b256 {
        __get_storage_key()
    }
}

storage {
    e1: Empty = Empty { },
    e2: Empty = Empty { },
}

abi GetStorageKeyTest {
    fn foo1() -> b256;
    fn foo2() -> b256;
}

impl GetStorageKeyTest for Contract {
    fn foo1() -> b256 {
        storage.e1.bar()
    }
    fn foo2() -> b256 {
        storage.e2.bar()
    }
}

// check: fn foo1<2994c98e>() -> b256
// check: entry():
// nextln: $(empty_struct_val=$VAL) = get_local {  } $ID
// nextln: $(retv=$VAL) = get_local b256 $(local_ret_var=$ID)
// nextln: call $(fn_name=$ID)($empty_struct_val, $retv)

// check: fn $fn_name(self $MD: {  }, inout __ret_value $MD: b256) -> b256
// nextln: entry(self: {  }
// nextln: $(key_val=$VAL) = get_storage_key
// nextln: mem_copy __ret_value, $key_val, 32
// nextln: ret b256 __ret_value
