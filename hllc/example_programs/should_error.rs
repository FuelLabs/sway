struct MyStruct {
    field_a: u64,
    field_b: bool, // note the trailing comma with no following field
}

fn main() {
    let x: MyStruct = new_mystruct();
    
    x.do_thing();

}
