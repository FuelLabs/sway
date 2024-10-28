contract;

configurable {
    CONFIG: u64 = 0,
}

pub fn test() {
    let x = 0;
    let y = 0;

    // Shadowing a variable.

    let _ = asm(x) { // Not used.
        zero
    };

    let _ = asm(x) { // Used.
        movi x i0;
    };

    let _ = asm(x: 0) {
        zero
    };

    let _ = asm(x: x) {
        zero
    };

    let _ = asm(x: y) {
        zero
    };

    // Shadowing a configurable.

    let _ = asm(CONFIG) { // Not used.
        zero
    };

    let _ = asm(CONFIG) { // Used.
        movi CONFIG i0;
    };

    let _ = asm(CONFIG: 0) {
        zero
    };

    let _ = asm(CONFIG: CONFIG) {
        zero
    };

    let _ = asm(CONFIG: y) {
        zero
    };

    // Shadowing a non-local constant.

    let _ = asm(G_CONST) { // Not used.
        zero
    };

    let _ = asm(G_CONST) { // Used.
        movi G_CONST i0;
    };

    let _ = asm(G_CONST: 0) {
        zero
    };

    let _ = asm(G_CONST: G_CONST) {
        zero
    };

    let _ = asm(G_CONST: y) {
        zero
    };

    const L_CONST: u64 = 0;

    // Shadowing a local constant.

    let _ = asm(L_CONST) { // Not used.
        zero
    };

    let _ = asm(L_CONST) { // Used.
        movi L_CONST i0;
    };

    let _ = asm(L_CONST: 0) {
        zero
    };

    let _ = asm(L_CONST: L_CONST) {
        zero
    };

    let _ = asm(L_CONST: y) {
        zero
    };
}

const G_CONST: u64 = 0;