script;

const MY_CUSTOM_ERROR_MESSAGE: u64 = 100;
const forty_twos: b256 = 0x4242424242424242424242424242424242424242424242424242424242424242;

struct CustomError {
    val_1: bool,
    val_2: b256,
    val_3: u64,
}

fn main() -> bool {
    let a = 5;
    let b = 5;
    let c = 6;

    require(1 + 1 == 2, 11);
    require(true == true, 42);
    require(false != true, 0);
    require(7 > 5, 3);
    require(a == b, MY_CUSTOM_ERROR_MESSAGE);
    require(c == 6, CustomError {
        val_1: false,
        val_2: forty_twos,
        val_3: 11,
    });

    true
}
