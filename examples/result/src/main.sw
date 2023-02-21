script;

enum MyContractError {
    DivisionByZero: (),
}

fn divide(numerator: u64, denominator: u64) -> Result<u64, MyContractError> {
    if (denominator == 0) {
        return Result::Err(MyContractError::MyErrorMessage);
    } else {
        Result::Ok(numerator / denominator)
    }
}

fn main() -> Result<u64, str[4]> {
    let result = divide(20, 2);
    match result {
        Result::Ok(value) => Result::Ok(value),
        Result::Err(MyContractError::DivisionByZero) => Result::Err("Fail"),
    }
}
