script;

fn my_function(foo: u64, bar: u64, long_argument_name: u64) -> u64 {
    foo + bar + long_argument_name
}

fn main() {
    let x = my_function(1, 2, 3);
    let foo = 1;
    let y = my_function(foo, 2, 3);
    let bar = 2;
    let z = my_function(foo, bar, 3);
    let long_argument_name = 3;
    let w = my_function(foo, bar, long_argument_name);
}