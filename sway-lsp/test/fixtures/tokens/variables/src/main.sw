script;

struct ExampleStruct {
    variable: u32,
}

enum ExampleEnum {
    Variants: u32,
}

// Function parameters
fn example_function(variable: u32) -> u32 {
    variable
}

enum Result<T, E> {
    Ok: T,
    Err: E,
}

fn main() {
    // Variable usage: Variable Declarations
    let variable1 = 10;
    let variable2 = variable1;
    let variable3 = false;
    let variable4 = "test";

    // Variable usage: Function arguments
    let _ = example_function(variable2);

    // Variable usage: Struct fields
    let _ = ExampleStruct { variable: variable1 };

    // Variable usage: Enum variants
    let _ = ExampleEnum::Variants(variable1);

    // Variable usage: Tuple elements
    let _ = (variable3, 20);

    // Variable usage: Array elements
    let _ = [variable4, 20];

    // Variable usage: Scoped Declarations
    {
        let variable1 = 1234;
        log(variable1);
    }

    // Variable usage: If let scopes
    let x: Result<u64, u64> = Result::Ok::<u64, u64>(5u64);
    let variable3 = if let Result::Ok(y) = x { y + 10 } else { 1 };
    
    // Variable usage: Shadowing
    let variable5 = variable3; 
}