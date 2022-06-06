library tuples;

fn tuple() {
    // You can declare the types youself
    let tuple1: (u8, bool, u64) = (100, false, 10000);

    // Or have the types be inferred
    let tuple2 = (5, true, ("Sway", 8));

    // Retrieve values from tuples
    let number = tuple1.0;
    let sway = tuple2.2.1;

    // Destructure the values from the tuple into variables
    let(n1, truthness, n2) = tuple1;

    // If you do not care about specific values then use "_"
    let(_, truthness, _) = tuple2;
}
