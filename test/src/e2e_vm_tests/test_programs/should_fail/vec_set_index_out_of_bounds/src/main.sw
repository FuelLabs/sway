script;

fn main() {
    let mut vector = Vec::new();

    let number0 = 0u8;
    let number1 = 1u8;

    vector.push(number0);
    vector.push(number1);

    assert(vector.len() == 2);
    assert(vector.capacity() == 2);
    assert(vector.is_empty() == false);

    vector.set(3, 5);
}
