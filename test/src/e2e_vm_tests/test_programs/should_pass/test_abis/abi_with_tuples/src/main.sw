library abi_with_tuples;

pub struct Person {
    age: u64
}

pub enum Location {
    Earth: ()
}


abi MyContract {
    fn bug1(param: (Person, u64)) -> bool;
    fn bug2(param: (Location, u64)) -> bool;
}