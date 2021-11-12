pub(crate) fn is_snake_case(ident: &str) -> bool {
    ident.chars().all(|c| match c {
        '_' => true,
        c if !c.is_uppercase() => true,
        _ => false,
    }) && !ident.contains("__")
}

#[test]
fn test_case_is_snake_case() {
    assert!(is_snake_case(""));
    assert!(is_snake_case("foo"));
    assert!(is_snake_case("foo_bar"));
    assert!(is_snake_case("_foo_bar"));
    assert!(is_snake_case("foo_bar_"));
    assert!(is_snake_case("_foo_bar_"));

    assert!(is_snake_case("foo1"));
    assert!(is_snake_case("foo1_b1ar"));
    assert!(is_snake_case("_1foo_bar"));
    assert!(is_snake_case("_1_")); // In Sway syntax this is a number.
    assert!(is_snake_case("_foo_1_"));

    assert!(!is_snake_case("Foo"));
    assert!(!is_snake_case("fOo"));
    assert!(!is_snake_case("foo_Bar"));
    assert!(!is_snake_case("foo_baR"));
    assert!(!is_snake_case("foo__bar"));
    assert!(!is_snake_case("FOO_BAR_BAZ"));
    assert!(!is_snake_case("_FOO_BAR_BAZ"));
}

pub(crate) fn is_pascal_case(ident: &str) -> bool {
    let ident = ident.trim_start_matches('_');
    (match ident.chars().next() {
        None => true,
        Some(c) => c.is_uppercase(),
    }) && !ident.contains('_')
}

#[test]
fn test_case_is_pascal_case() {
    assert!(is_pascal_case(""));
    assert!(is_pascal_case("Foo"));
    assert!(is_pascal_case("FooB"));
    assert!(is_pascal_case("FooBar"));
    assert!(is_pascal_case("FooBarBaz"));

    assert!(is_pascal_case("Foo1"));
    assert!(is_pascal_case("F00B"));
    assert!(is_pascal_case("F00b"));
    assert!(is_pascal_case("FooB4r"));
    assert!(is_pascal_case("FooBarBaz1"));

    assert!(is_pascal_case("FooBA"));
    assert!(is_pascal_case("FOOBA")); // Rust likes it so... so do we.

    assert!(!is_pascal_case("foo"));
    assert!(!is_pascal_case("foo_bar"));
    assert!(!is_pascal_case("_foo_bar"));

    assert!(is_pascal_case("_Foo"));
    assert!(is_pascal_case("_FooBar"));
    assert!(!is_pascal_case("fOOBA"));
    assert!(!is_pascal_case("_fOOBA"));
}

pub(crate) fn to_snake_case(ident: &str) -> String {
    // Simultaneously trim and preserve any leading underscores.
    let mut prefix = String::new();
    let ident = ident.trim_start_matches(|c: char| {
        if c == '_' {
            prefix.push(c);
            true
        } else {
            false
        }
    });

    // Split the string into any underscores separated parts and process them separately, then join
    // them again.
    let mut words = Vec::new();
    for part in ident.split('_') {
        if part.is_empty() {
            continue;
        }

        // Loop for each character.  When we come accross a 'boundary' of lowercase followed by
        // uppercase then save the characters to that point in a word and start a new word.
        let (mut sub_parts, final_part, _) = part.chars().fold(
            (Vec::new(), String::new(), false),
            |(mut words, mut cur_word, prev_upper), c| {
                if !prev_upper && c.is_uppercase() {
                    // A boundary; start a new word.
                    if !cur_word.is_empty() {
                        words.push(cur_word);
                    }
                    (words, c.to_lowercase().collect(), true)
                } else {
                    // Prev was upper; push this char to current word.
                    cur_word.extend(c.to_lowercase());
                    (words, cur_word, c.is_uppercase())
                }
            },
        );
        sub_parts.push(final_part);
        words.push(sub_parts.join("_"));
    }
    prefix + &words.join("_")
}

#[test]
fn test_case_to_snake_case() {
    assert_eq!(to_snake_case("Foo"), "foo");
    assert_eq!(to_snake_case("FooB"), "foo_b");
    assert_eq!(to_snake_case("FooBar"), "foo_bar");
    assert_eq!(to_snake_case("FooBarBaz"), "foo_bar_baz");

    assert_eq!(to_snake_case("Foo1"), "foo1");
    assert_eq!(to_snake_case("F00B"), "f00_b");
    assert_eq!(to_snake_case("F00b"), "f00b");
    assert_eq!(to_snake_case("FooB4r"), "foo_b4r");
    assert_eq!(to_snake_case("FooBarBaz1"), "foo_bar_baz1");

    assert_eq!(to_snake_case("FooBA"), "foo_ba");
    assert_eq!(to_snake_case("FOOBA"), "fooba");

    assert_eq!(to_snake_case("foo"), "foo");
    assert_eq!(to_snake_case("foo_bar"), "foo_bar");
    assert_eq!(to_snake_case("_foo_bar"), "_foo_bar");

    assert_eq!(to_snake_case("_Foo"), "_foo");
    assert_eq!(to_snake_case("_FooBar"), "_foo_bar");
    assert_eq!(to_snake_case("__FooBar"), "__foo_bar");
    assert_eq!(to_snake_case("FooBar_FooBar"), "foo_bar_foo_bar");
    assert_eq!(to_snake_case("fOOBA"), "f_ooba");
    assert_eq!(to_snake_case("_fOOBA"), "_f_ooba");
    assert_eq!(to_snake_case("__fOOBA"), "__f_ooba");

    assert_eq!(to_snake_case("foo__bar"), "foo_bar");
}

pub(crate) fn to_pascal_case(ident: &str) -> String {
    // Simultaneously trim and preserve any leading underscores.
    let mut prefix = String::new();
    let ident = ident.trim_start_matches(|c: char| {
        if c == '_' {
            prefix.push(c);
            true
        } else {
            false
        }
    });

    // Split into underscore separated parts and rejoin with capitalisation.  It's actually easier
    // (though probably not that efficient) to convert this to snake case before recombining into
    // pascal case.
    let ident = to_snake_case(&ident);
    prefix
        + &ident
            .split('_')
            .map(|part| {
                let mut iter = part.chars();
                iter.next().unwrap().to_uppercase().chain(iter).collect()
            })
            .collect::<Vec<String>>()
            .join("")
}

#[test]
fn test_case_to_pascal_case() {
    assert_eq!(to_pascal_case("foo"), "Foo");
    assert_eq!(to_pascal_case("foo_bar"), "FooBar");
    assert_eq!(to_pascal_case("_foo_bar"), "_FooBar");
    assert_eq!(to_pascal_case("foo_bar_"), "FooBar");
    assert_eq!(to_pascal_case("_foo_bar_"), "_FooBar");

    assert_eq!(to_pascal_case("foo1"), "Foo1");
    assert_eq!(to_pascal_case("foo1_b1ar"), "Foo1B1ar");
    assert_eq!(to_pascal_case("_1foo_bar"), "_1fooBar"); // This is contentious.
    assert_eq!(to_pascal_case("_foo_1_"), "_Foo1");

    assert_eq!(to_pascal_case("Foo"), "Foo");
    assert_eq!(to_pascal_case("fOo"), "FOo");
    assert_eq!(to_pascal_case("fooBar"), "FooBar");
    assert_eq!(to_pascal_case("fooBar_fooBar"), "FooBarFooBar");
    assert_eq!(to_pascal_case("foo_Bar"), "FooBar");
    assert_eq!(to_pascal_case("foo_baR"), "FooBaR");
    assert_eq!(to_pascal_case("foo__bar"), "FooBar");
    assert_eq!(to_pascal_case("FOO_BAR_BAZ"), "FooBarBaz");
    assert_eq!(to_pascal_case("_FOO_BAR_BAZ"), "_FooBarBaz");
}
