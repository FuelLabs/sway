contract;

fn bar() -> bool {
    let x: u8 = 42y8; // Lexer recovery here.

    0 // recovery witness
}
