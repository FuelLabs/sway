library;

fn explicit() {
    // ANCHOR: explicit
    let fuel: str = "fuel";
    let blockchain: str = "blockchain";
    let crypto: str[6] = __to_str_array("crypto");
    // ANCHOR_END: explicit
}

fn implicit() {
    // ANCHOR: implicit
    // The variable `fuel` is a string slice with length equals 4
    let fuel = "fuel";
    let crypto = __to_str_array("crypto");
    // ANCHOR_END: implicit
}
