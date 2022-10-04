library strings;

fn explicit() {
    // ANCHOR: explicit
    let fuel: str[4] = "fuel";
    let blockchain: str[10] = "blockchain";
    let crypto: str[6] = "crypto";
    // ANCHOR_END: explicit
}

fn implicit() {
    // ANCHOR: implicit
    let fuel = "fuel";
    let blockchain = "blockchain";
    let crypto = "crypto";
    // ANCHOR_END: implicit
}

fn alternative_quotes() {
    // ANCHOR: alternative_quotes
    let fuel = 'fuel';
    let blockchain = 'blockchain';
    let crypto = 'crypto';
    // ANCHOR_END: alternative_quotes
}
