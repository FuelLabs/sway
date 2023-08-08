library;

fn explicit() {
    // ANCHOR: explicit
    let fuel: str[4] = "fuel";
    let blockchain: str[10] = "blockchain";
    let crypto: str[6] = "crypto";
    // ANCHOR_END: explicit
}

fn implicit() {
    // ANCHOR: implicit
    // The variable `fuel` has a length of 4
    let fuel = "fuel";
    // ANCHOR_END: implicit
}
