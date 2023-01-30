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
// nextln: $(empty_struct_val=$VAL) = get_local ptr {  }, $ID
// nextln: $(loaded=$VAL) = load $empty_struct_val
// nextln: call $(fn_name=$ID)($loaded)

// check: fn $fn_name(self $MD: {  }) -> b256
// nextln: entry(self: {  }
// nextln: $(key_val=$VAL) = get_storage_key
// nextln: ret b256 $key_val
