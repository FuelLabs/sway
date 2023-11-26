script;

fn main() -> bool {
    // Test with integers
    let option1 = Option::Some(10);
    let option2 = Option::Some(10);
    let option3 = Option::Some(20);
    let none_option: Option<u64> = Option::None;

    // Eq is True
    assert(option1 == option2);
    assert(option2 == option1);

    assert(none_option == Option::None);
    assert(Option::<u64>::None == none_option);

    // Eq is False
    assert(!(option1 == option3));
    assert(!(option3 == option1));

    assert(!(option1 == Option::<u64>::None));
    assert(!(Option::<u64>::None == option1));

    assert(!(none_option == option1));
    assert(!(option1 == none_option));

    // Neq is True
    assert(option1 != option3);
    assert(option3 != option1);

    assert(option1 != Option::<u64>::None);
    assert(Option::<u64>::None != option1);

    assert(none_option != option1);
    assert(option1 != none_option);

    // Neq is False
    assert(!(option1 != option2));
    assert(!(option2 != option1));

    assert(!(none_option != Option::<u64>::None));
    assert(!(Option::<u64>::None != none_option));

    // Test with booleans
    let bool_option1 = Option::Some(true);
    let bool_option2 = Option::Some(true);
    let bool_option3 = Option::Some(false);
    let none_bool_option: Option<bool> = Option::None;

    // Bool equality tests
    assert(bool_option1 == bool_option2);
    assert(bool_option2 == bool_option1);

    assert(bool_option1 == Option::Some(true));
    assert(Option::Some(true) == bool_option1);

    assert(bool_option2 == Option::Some(true));
    assert(Option::Some(true) == bool_option2);

    assert(!(bool_option3 == Option::Some(true)));
    assert(!(Option::Some(true) == bool_option3));

    // Bool None tests
    assert(none_bool_option == Option::<bool>::None);
    assert(Option::<bool>::None == none_bool_option);

    assert(!(bool_option1 == none_bool_option));
    assert(!(none_bool_option == bool_option1));

    assert(!(bool_option3 == none_bool_option));
    assert(!(none_bool_option == bool_option3));

    // Bool inequality tests
    assert(bool_option1 != bool_option3);
    assert(bool_option3 != bool_option1);

    assert(bool_option1 != none_bool_option);
    assert(none_bool_option != bool_option1);

    assert(!(bool_option1 != bool_option2));
    assert(!(bool_option2 != bool_option1));

    assert(!(none_bool_option != Option::<bool>::None));
    assert(!(Option::<bool>::None != none_bool_option));
    
    true
}
