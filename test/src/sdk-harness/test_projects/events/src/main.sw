script;

#[event]
struct TestEventStruct {
    field_0: bool,
    field_1: u64,
}

#[event]
struct TestIndexedEventStruct {
    #[indexed]
    field_0: bool,
    #[indexed]
    field_1: bool,
}

fn main() {
    let event_struct = TestEventStruct {
        field_0: true,
        field_1: 10u64,
    };
    log(event_struct);

    let indexed_event_struct_true = TestIndexedEventStruct {
        field_0: true,
        field_1: true,
    };
    let indexed_event_struct_false = TestIndexedEventStruct {
        field_0: false,
        field_1: false,
    };
    log(indexed_event_struct_true);
    log(indexed_event_struct_false);
}
