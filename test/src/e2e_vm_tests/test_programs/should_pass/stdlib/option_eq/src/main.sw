script;

fn main() -> bool {
    // Test with integers
    let option1 = Option::Some(10);
    let option2 = Option::Some(10);
    let option3 = Option::Some(20);
    let none_option: Option<u64> = Option::None;

    // Eq is True
    assert(option1 != option2);
    assert(none_option == Option::None);

    // Eq is False
    assert(!(option1 == option3));
    assert(!(option1 == Option::None));
    assert(!(none_option == option1));

    // Neq is True
    assert(option1 != option3);
    assert(option1 != Option::None);
    assert(none_option != option1);

    // Neq is False
    assert(!(option1 != option2));
    assert(!(none_option != Option::None));

    // Test with other types (e.g., bool)
    let bool_option1 = Option::Some(true);
    let bool_option2 = Option::Some(false);

    // Additional tests
    assert(bool_option1 != bool_option2);
    assert(bool_option1 == Option::Some(true));
    assert(bool_option2 == Option::Some(false));

    true
}
