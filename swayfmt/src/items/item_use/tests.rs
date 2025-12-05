use forc_diagnostic::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_item;

fmt_test_item!(multiline     "use foo::{
    quux,
    xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
    yxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
};",
          out_of_order  "use foo::{yxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx, quux, xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx};"
);

fmt_test_item!(multiline_with_trailing_comma     "use foo::{
    quux,
    xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
    yxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
};",
          out_of_order  "use foo::{yxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx, quux, xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,};"
);

fmt_test_item!(multiline_nested      "use foo::{
    Quux::{
        a,
        b,
        C,
    },
    xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
    yxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
};",
          out_of_order          "use foo::{xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx, Quux::{b, a, C}, yxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx};"
);

fmt_test_item!(multiline_nested_with_trailing_comma      "use foo::{
    Quux::{
        a,
        b,
        C,
    },
    xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
    yxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
};",
          out_of_order          "use foo::{xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx, Quux::{b, a, C,}, yxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,};"
);

fmt_test_item!(single_line_sort  "use foo::{bar, baz, Quux::{a, b, C}};",
          out_of_order      "use foo::{baz, Quux::{b, a, C}, bar};"
);

fmt_test_item!(single_line_sort_with_trailing_comma  "use foo::{bar, baz, Quux::{a, b, C}};",
          out_of_order      "use foo::{baz, Quux::{b, a, C,}, bar,};"
);

fmt_test_item!(single_import_without_braces      "use std::tx::tx_id;",
          braced_single_import      "use std::tx::{tx_id};"
);

fmt_test_item!(single_import_without_braces_with_trailing_comma      "use std::tx::tx_id;",
          braced_single_import      "use std::tx::{tx_id,};"
);

fmt_test_item!(single_import_multiline_with_braces      "use std::tx::{
    xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
};",
          braced_single_import      "use std::tx::{xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx};"
);

fmt_test_item!(single_import_multiline_with_braces_with_trailing_comma      "use std::tx::{
    xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,
};",
          braced_single_import      "use std::tx::{xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx,};"
);

#[test]
fn single_import_with_braces_preserves_following_item() {
    // Regression test for: https://github.com/FuelLabs/sway/issues/7434
    use crate::Formatter;
    use indoc::indoc;

    let unformatted = indoc! {r#"
        contract;
        use utils::{IssuanceParams};
        pub struct BridgeRegisteredEvent {
            pub bridge_name: String,
            pub bridge_id: b256,
        }
    "#};

    let expected = indoc! {r#"
        contract;
        use utils::IssuanceParams;
        pub struct BridgeRegisteredEvent {
            pub bridge_name: String,
            pub bridge_id: b256,
        }
    "#};

    let mut formatter = Formatter::default();
    let first_formatted = Formatter::format(&mut formatter, unformatted.into()).unwrap();

    // The critical assertion: "pub struct BridgeRegisteredEvent" should stay on one line
    assert!(
        !first_formatted.contains("pub struct\n"),
        "Bug regression: struct name was split from 'pub struct' keyword"
    );

    assert_eq!(first_formatted, expected);

    // Ensure idempotency
    let second_formatted =
        Formatter::format(&mut formatter, first_formatted.as_str().into()).unwrap();
    assert_eq!(second_formatted, first_formatted);
}
