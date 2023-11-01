script;

mod lib_literal;
mod lib_struct;
mod lib_enum;
mod lib_tuple;
mod lib_nested;

fn main() -> u64 {
    assert(::lib_literal::test() == 42);

    assert(::lib_struct::test() == 42);

    assert(::lib_enum::test() == 42);
    
    assert(::lib_tuple::test() == 42);
    
    assert(::lib_nested::test() == 42);

    42
}
