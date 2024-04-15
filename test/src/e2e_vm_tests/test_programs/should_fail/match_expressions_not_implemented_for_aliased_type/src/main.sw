script;

// This test proves that https://github.com/FuelLabs/sway/issues/5846 is fixed.

type ArrayAlias = [u64;3];

fn main() {
    let a = [1u64, 2u64, 3u64];
    array(a);
    array_alias(a);
}

fn array(array: [u64;3]) {
    let _ = match array {
        _ => true,
    };
}

fn array_alias(array_alias: ArrayAlias) {
    let _ = match array_alias {
        _ => true,
    };
}