library;

struct MyStruct {
    x: u64
}

// OK
const A: MyStruct = MyStruct {
    x: {
        1
    }
};

// NOK
const B: MyStruct = MyStruct {
    x: {
        return 1;
        1
    }
};
