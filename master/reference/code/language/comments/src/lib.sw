library;

fn comment() {
    // ANCHOR: comment
    // imagine that this line is twice as long
    // and it needed to be split onto multiple lines
    let baz = 8; // Eight is a good number
    // ANCHOR_END: comment
}

fn block() {
    // ANCHOR: block
    /*
        imagine that this line is twice as long
        and it needed to be split onto multiple lines
    */
    let baz = 8; /* Eight is a good number */ // ANCHOR_END: block
}

// ANCHOR: documentation
/// Data structure containing metadata about product XYZ
struct Product {
    /// Some information about field 1
    field1: u64,
    /// Some information about field 2
    field2: bool,
}

/// Creates a new instance of a Product
///
/// # Arguments
///
/// - `field1`: description of field1
/// - `field2`: description of field2
///
/// # Returns
///
/// A struct containing metadata about a Product
fn create_product(field1: u64, field2: bool) -> Product {
    Product { field1, field2 }
}
// ANCHOR_END: documentation
