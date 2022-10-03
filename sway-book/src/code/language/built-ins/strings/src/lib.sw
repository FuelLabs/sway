library strings;

// ANCHOR: explicit
fn explicit() {
    let fuel: str[4] = "fuel";
    let blockchain: str[10] = "blockchain";
    let crypto: str[6] = "crypto";
}
// ANCHOR_END: explicit

// ANCHOR: implicit
fn implicit() {
    let fuel = "fuel";
    let blockchain = "blockchain";
    let crypto = "crypto";
}
// ANCHOR_END: implicit

// ANCHOR: alternative_quotes
fn alternative_quotes() {
    let fuel = 'fuel';
    let blockchain = 'blockchain';
    let crypto = 'crypto';
}
// ANCHOR_END: alternative_quotes
