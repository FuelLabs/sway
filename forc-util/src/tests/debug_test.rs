use forc_util::debug::debug_tuple;

#[test]
fn test_debug_tuple() {
    let my_tuple = (42, "Fuel", 3.14);
    debug_tuple(&my_tuple);
    
}
