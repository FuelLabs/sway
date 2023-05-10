script;

enum MyContractError {
    DivisionByZero: (),
}

fn divide(numerator: u64, denominator: u64) -> Result<u64, MyContractError> {
    if (denominator == 0) {
        return Err(MyContractError::DivisionByZero);
    } else {
        Ok(numerator / denominator)
    }
}

fn main() -> Result<u64, str[4]> {
    let result = divide(20, 2);
    match result {
        Ok(value) => Ok(value),
        Err(MyContractError::DivisionByZero) => Err("Fail"),
    }
}
