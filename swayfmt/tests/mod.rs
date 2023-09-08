use std::sync::Arc;
use swayfmt::{config::user_def::FieldAlignment, Formatter};
use test_macros::assert_eq_pretty;

/// Takes a configured formatter as input and formats a given input and checks the actual output against an
/// expected output. There are two format passes to ensure that the received output does not change on a second pass.
fn check_with_formatter(unformatted: &str, expected: &str, formatter: &mut Formatter) {
    let first_formatted = Formatter::format(formatter, Arc::from(unformatted), None).unwrap();
    assert_eq_pretty!(first_formatted, expected);

    let second_formatted =
        Formatter::format(formatter, Arc::from(first_formatted.clone()), None).unwrap();
    assert_eq_pretty!(second_formatted, first_formatted);
}

/// Formats a given input and checks the actual output against an expected
/// output by calling check_with_formatter() with a default Formatter as input.
fn check(unformatted: &str, expected: &str) {
    let mut formatter = Formatter::default();
    check_with_formatter(unformatted, expected, &mut formatter);
}

#[test]
fn module_doc_comments_persist() {
    check(
        r#"

//! this is a module level doc comment
library;
        "#,
        r#"//! this is a module level doc comment
library;
"#,
    )
}

#[test]
fn conserve_pub_mod() {
    check(
        r#"contract;
pub mod foo;
"#,
        r#"contract;
pub mod foo;
"#,
    )
}

#[test]
fn const_spacing() {
    check(
        r#"contract;
pub const TEST:u16=10;
"#,
        r#"contract;
pub const TEST: u16 = 10;
"#,
    )
}

#[test]
fn struct_alignment() {
    let mut formatter = Formatter::default();
    formatter.config.structures.field_alignment = FieldAlignment::AlignFields(40);

    check_with_formatter(
        r#"contract;
pub struct Foo<T, P> {
   barbazfoo: u64,
   baz  : bool,
}
"#,
        r#"contract;
pub struct Foo<T, P> {
    barbazfoo : u64,
    baz       : bool,
}
"#,
        &mut formatter,
    );
}
#[test]
fn struct_ending_comma() {
    check(
        r#"contract;
pub struct Foo {
    bar: u64,
    baz: bool
}
"#,
        r#"contract;
pub struct Foo {
    bar: u64,
    baz: bool,
}
"#,
    );
}

#[test]
fn enum_without_variant_alignment() {
    check(
        r#"contract;

enum Color {
    Blue: (), Green: (),
            Red: (),
    Silver: (),
                    Grey: () }
        "#,
        r#"contract;

enum Color {
    Blue: (),
    Green: (),
    Red: (),
    Silver: (),
    Grey: (),
}
"#,
    );
}
#[test]
fn enum_with_variant_alignment() {
    // Creating a config with enum_variant_align_threshold that exceeds longest variant length
    let mut formatter = Formatter::default();
    formatter.config.structures.field_alignment = FieldAlignment::AlignFields(20);
    check_with_formatter(
        r#"contract;

enum Color {
    Blue: (), Green: (),
            Red: (),
    Silver: (),
                    Grey: (), }
        "#,
        r#"contract;

enum Color {
    Blue   : (),
    Green  : (),
    Red    : (),
    Silver : (),
    Grey   : (),
}
"#,
        &mut formatter,
    );
}
#[test]
fn item_abi_with_generics_and_attributes() {
    check(
        r#"contract;

abi StorageMapExample {
    #[storage(write)]fn insert_into_map1(key: u64, value: u64);

fn hello(key: u64, value: u64);
}"#,
        r#"contract;

abi StorageMapExample {
    #[storage(write)]
    fn insert_into_map1(key: u64, value: u64);

    fn hello(key: u64, value: u64);
}
"#,
    );
}

#[test]
fn multi_items() {
    check(
        r#"contract;

pub const TEST: u16 = 10;
pub const TEST1: u16 = 10;"#,
        r#"contract;

pub const TEST: u16 = 10;
pub const TEST1: u16 = 10;
"#,
    );
}
#[test]
fn ty_formatting() {
    check(
        r#"contract;

enum TestTy {
    Infer:
    _,
    Array : [u8;
    40],
    String:         str[
    4
    ],
    PathType     : root::
example::
    some_type,
    TupleNil: (),
    Tuple: (   u64,
        u32
    ),
}"#,
        r#"contract;

enum TestTy {
    Infer: _,
    Array: [u8; 40],
    String: str[4],
    PathType: root::example::some_type,
    TupleNil: (),
    Tuple: (u64, u32),
}
"#,
    );
}
#[test]
fn storage_without_alignment() {
    check(
        r#"contract;

        struct Type1 {
            foo: u64,
        }

        struct Type2 {
            bar: u64,
        }

        storage {
         var1: Type1=Type1{ foo: 8 },
              var2: Type2=Type2{ bar: 9 },
        }
        "#,
        r#"contract;

struct Type1 {
    foo: u64,
}

struct Type2 {
    bar: u64,
}

storage {
    var1: Type1 = Type1 { foo: 8 },
    var2: Type2 = Type2 { bar: 9 },
}
"#,
    );
}
#[test]
fn storage_with_alignment() {
    let mut formatter = Formatter::default();
    formatter.config.structures.field_alignment = FieldAlignment::AlignFields(50);
    check_with_formatter(
        r#"contract;

struct Type1 {
    foo: u64,
}

struct Type2 {
    bar: u64,
}

storage {
 long_var_name: Type1=Type1{ foo: 8 },
      var2: Type2=Type2{ bar: 9 },
}
"#,
        r#"contract;

struct Type1 {
    foo : u64,
}

struct Type2 {
    bar : u64,
}

storage {
    long_var_name : Type1 = Type1 { foo: 8 },
    var2          : Type2 = Type2 { bar: 9 },
}
"#,
        &mut formatter,
    );
}
#[test]
fn storage_initializer() {
    check(
        r#"contract;

struct Type1 {
    x: u64,
    y: u64,
}

struct Type2 {
    w: b256,
    z: bool,
}

storage {
    var1: Type1 = Type1 {

        x: 0,

        y:
        0,
        },
    var2: Type2 = Type2 { w: 0x0000000000000000000000000000000000000000000000000000000000000000,z: false,
    },
}"#,
        r#"contract;

struct Type1 {
    x: u64,
    y: u64,
}

struct Type2 {
    w: b256,
    z: bool,
}

storage {
    var1: Type1 = Type1 { x: 0, y: 0 },
    var2: Type2 = Type2 {
        w: 0x0000000000000000000000000000000000000000000000000000000000000000,
        z: false,
    },
}
"#,
    );
}
#[test]
fn item_fn() {
    check(
        r#"contract;

pub fn hello( person: String ) -> String {let greeting = 42;greeting.to_string()}
fn goodbye() -> usize {let farewell: usize = 5; farewell }"#,
        r#"contract;

pub fn hello(person: String) -> String {
    let greeting = 42;
    greeting.to_string()
}
fn goodbye() -> usize {
    let farewell: usize = 5;
    farewell
}
"#,
    );
}
#[test]
fn same_line_where() {
    check(
        r#"contract;

pub fn hello( person: String ) -> String where T: Eq,{let greeting = 42;greeting.to_string()}"#,
        r#"contract;

pub fn hello(person: String) -> String
where
    T: Eq,
{
    let greeting = 42;
    greeting.to_string()
}
"#,
    );
}
#[test]
fn trait_and_super_trait() {
    check(
        r#"library;

trait Person{ fn name( self )->String;fn age( self )->usize; }
trait Student:Person {fn university(self) -> String;}
trait Programmer {fn fav_language(self) -> String;}
trait CompSciStudent: Programmer+Student {fn git_username(self) -> String;}
trait TraitWithGenerics<T> where T: String {fn from(b: T) -> Self;}"#,
        r#"library;

trait Person {
    fn name(self) -> String;
    fn age(self) -> usize;
}
trait Student: Person {
    fn university(self) -> String;
}
trait Programmer {
    fn fav_language(self) -> String;
}
trait CompSciStudent: Programmer + Student {
    fn git_username(self) -> String;
}
trait TraitWithGenerics<T>
where
    T: String
{
    fn from(b: T) -> Self;
}
"#,
    );
}
#[test]
fn method_calls() {
    let mut formatter = Formatter::default();
    formatter.config.structures.small_structures_single_line = true;
    formatter.config.whitespace.max_width = 220;
    check_with_formatter(
        r#"script;

struct Opts {
    gas: u64,
    coins: u64,
    id: ContractId,
}

fn  main(       ) -> bool{
    let default_gas  = 1_000_000_000_000           ;let fuelcoin_id = ContractId::from(0x018f59fe434b323a5054e7bb41de983f4926a3c5d3e4e1f9f33b5f0f0e611889);

    let balance_test_id = ContractId :: from( 0x597e5ddb1a6bec92a96a73e4f0bc6f6e3e7b21f5e03e1c812cd63cffac480463 ) ;

    let fuel_coin = abi(    TestFuelCoin, fuelcoin_id.into(       ) ) ;

    assert(fuelcoin_balance == 0);

    fuel_coin.mint        {
        gas:             default_gas
    }

    (11);

    fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    assert( fuelcoin_balance   == 11 ) ;

    fuel_coin.burn {
        gas: default_gas
    }
    (7);

    fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    assert(fuelcoin_balance == 4);

    fuel_coin.force_transfer {
        gas: default_gas
    }
    (3, fuelcoin_id, balance_test_id);

    fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    let balance_test_contract_balance = balance_of(fuelcoin_id, balance_test_id);
    assert(fuelcoin_balance == 1);
    assert(balance_test_contract_balance == 3);

    true
}"#,
        r#"script;

struct Opts {
    gas: u64,
    coins: u64,
    id: ContractId,
}

fn main() -> bool {
    let default_gas = 1_000_000_000_000;
    let fuelcoin_id = ContractId::from(0x018f59fe434b323a5054e7bb41de983f4926a3c5d3e4e1f9f33b5f0f0e611889);

    let balance_test_id = ContractId::from(0x597e5ddb1a6bec92a96a73e4f0bc6f6e3e7b21f5e03e1c812cd63cffac480463);

    let fuel_coin = abi(TestFuelCoin, fuelcoin_id.into());

    assert(fuelcoin_balance == 0);

    fuel_coin.mint { gas: default_gas }(11);

    fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    assert(fuelcoin_balance == 11);

    fuel_coin.burn { gas: default_gas }(7);

    fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    assert(fuelcoin_balance == 4);

    fuel_coin.force_transfer { gas: default_gas }(3, fuelcoin_id, balance_test_id);

    fuelcoin_balance = balance_of(fuelcoin_id, fuelcoin_id);
    let balance_test_contract_balance = balance_of(fuelcoin_id, balance_test_id);
    assert(fuelcoin_balance == 1);
    assert(balance_test_contract_balance == 3);

    true
}
"#,
        &mut formatter,
    );
}

#[test]
fn struct_comments() {
    check(
        r"contract;
// This is a comment, for this one to be placed correctly we need to have Module visitor implemented
pub struct Foo { // Here is a comment



    // Trying some ASCII art
    baz:u64,




    bazzz:u64
             //  ________ ___  ___  _______   ___               ___       ________  ________  ________
             // |\  _____\\  \|\  \|\  ___ \ |\  \             |\  \     |\   __  \|\   __  \|\   ____\
             // \ \  \__/\ \  \\\  \ \   __/|\ \  \            \ \  \    \ \  \|\  \ \  \|\ /\ \  \___|_
             //  \ \   __\\ \  \\\  \ \  \_|/_\ \  \            \ \  \    \ \   __  \ \   __  \ \_____  \
             //   \ \  \_| \ \  \\\  \ \  \_|\ \ \  \____        \ \  \____\ \  \ \  \ \  \|\  \|____|\  \
             //    \ \__\   \ \_______\ \_______\ \_______\       \ \_______\ \__\ \__\ \_______\____\_\  \
             //     \|__|    \|_______|\|_______|\|_______|        \|_______|\|__|\|__|\|_______|\_________\
             //                                                                                  \|_________|
}
",
        r"contract;
// This is a comment, for this one to be placed correctly we need to have Module visitor implemented
pub struct Foo { // Here is a comment

    // Trying some ASCII art
    baz: u64,
    bazzz: u64,
    //  ________ ___  ___  _______   ___               ___       ________  ________  ________
    // |\  _____\\  \|\  \|\  ___ \ |\  \             |\  \     |\   __  \|\   __  \|\   ____\
    // \ \  \__/\ \  \\\  \ \   __/|\ \  \            \ \  \    \ \  \|\  \ \  \|\ /\ \  \___|_
    //  \ \   __\\ \  \\\  \ \  \_|/_\ \  \            \ \  \    \ \   __  \ \   __  \ \_____  \
    //   \ \  \_| \ \  \\\  \ \  \_|\ \ \  \____        \ \  \____\ \  \ \  \ \  \|\  \|____|\  \
    //    \ \__\   \ \_______\ \_______\ \_______\       \ \_______\ \__\ \__\ \_______\____\_\  \
    //     \|__|    \|_______|\|_______|\|_______|        \|_______|\|__|\|__|\|_______|\_________\
    //                                                                                  \|_________|
}
",
    );
}

#[test]
fn comments_empty_struct() {
    check(
        r#"contract;

struct AlignMyComments {
    // Align here please
        // Overindented comment
// Underindented comment
}"#,
        r#"contract;

struct AlignMyComments {
    // Align here please
    // Overindented comment
    // Underindented comment
}
"#,
    );
}

#[test]
fn comments_empty_traits() {
    check(
        r#"contract;

trait AlignMyComments {
    // Align here please
        // Overindented comment
// Underindented comment
}"#,
        r#"contract;

trait AlignMyComments {
    // Align here please
    // Overindented comment
    // Underindented comment
}
"#,
    );
}

#[test]
fn comments_empty_fns() {
    check(
        r#"contract;

fn single_comment_same_line() { /* a comment */ }

fn single_comment_same_line_trailing() {  // a comment
}

fn single_comment() -> bool {
    // TODO: This is a TODO
}

fn multiline_comments() {
    // Multi
        // line
// comment
}"#,
        r#"contract;

fn single_comment_same_line() { /* a comment */ }

fn single_comment_same_line_trailing() { // a comment
}

fn single_comment() -> bool {
    // TODO: This is a TODO
}

fn multiline_comments() {
    // Multi
    // line
    // comment
}
"#,
    );
}

#[test]
fn enum_comments() {
    check(
        r#"contract;
pub enum Bazz { // Here is a comment
    // Trying some ASCII art
    baz: (),





    bazzz: (),//-----
              //--D--
              //-----
}
"#,
        r#"contract;
pub enum Bazz { // Here is a comment
    // Trying some ASCII art
    baz: (),
    bazzz: (), //-----
    //--D--
    //-----
}
"#,
    );
}

#[test]
fn fn_comments() {
    check(
        r#"contract;
// This is a comment before a fn
// This is another comment before a fn
fn hello_world( baz: /* this is a comment */ u64) { let x = 5; // This is a comment inside the block
}
"#,
        r#"contract;
// This is a comment before a fn
// This is another comment before a fn
fn hello_world(baz: /* this is a comment */ u64) {
    let x = 5; // This is a comment inside the block
}
"#,
    );
}

#[test]
fn abi_comments() {
    check(
        r#"contract;

// This is an abi
abi StorageMapExample {
    // insert_into_map is blah blah
    #[storage(write)] // this is some other comment
    fn insert_into_map(key: u64, value: u64);
    // this is the last comment inside the StorageMapExample
}

// This is another abi
abi AnotherAbi {
    // insert_into_map is blah blah
    #[storage(write)]
    fn update_map(key: u64, value: u64);
        // this is some other comment
    fn read(key: u64);
}

abi CommentsInBetween {
    fn foo();
    // This should not collapse below

    // this is a comment
    fn bar();
}

// This is another abi
abi Empty {
    // Empty abi
}
"#,
        r#"contract;

// This is an abi
abi StorageMapExample {
    // insert_into_map is blah blah
    #[storage(write)] // this is some other comment
    fn insert_into_map(key: u64, value: u64);
    // this is the last comment inside the StorageMapExample
}

// This is another abi
abi AnotherAbi {
    // insert_into_map is blah blah
    #[storage(write)]
    fn update_map(key: u64, value: u64);
    // this is some other comment
    fn read(key: u64);
}

abi CommentsInBetween {
    fn foo();
    // This should not collapse below

    // this is a comment
    fn bar();
}

// This is another abi
abi Empty {
    // Empty abi
}
"#,
    );
}

#[test]
fn const_comments() {
    check(
        r#"contract;
pub const /* TEST: blah blah tests */ TEST: u16 = 10; // This is a comment next to a const"#,
        r#"contract;
pub const /* TEST: blah blah tests */ TEST: u16 = 10; // This is a comment next to a const
"#,
    );
}
#[test]
fn storage_comments() {
    check(
        r#"contract;

struct Type1 {
    foo: u64,
}
struct Type2 {
    bar: u64,
}
storage {
    // Testing a comment inside storage
    long_var_name: Type1=Type1{ foo: 8},
    // Testing another comment
    var2: Type2 = Type2{bar:9} // This is the last comment
}"#,
        r#"contract;

struct Type1 {
    foo: u64,
}
struct Type2 {
    bar: u64,
}
storage {
    // Testing a comment inside storage
    long_var_name: Type1 = Type1 { foo: 8 },
    // Testing another comment
    var2: Type2 = Type2 { bar: 9 }, // This is the last comment
}
"#,
    );
}

#[test]
fn trait_comments() {
    check(
        r#"contract;
// This is the programmer trait
trait Programmer {
    // Returns fav languages of this Programmer.
    fn fav_language(self) -> String;
}"#,
        r#"contract;
// This is the programmer trait
trait Programmer {
    // Returns fav languages of this Programmer.
    fn fav_language(self) -> String;
}
"#,
    );
}

#[test]
fn where_comment() {
    check(
        r#"contract;

pub fn hello( person: String ) -> String where /* This is next to where */ T: Eq, /*Here is a comment*/{let greeting = 42;greeting.to_string()}"#,
        r#"contract;

pub fn hello(person: String) -> String
where /* This is next to where */
    T: Eq, /*Here is a comment*/
{
    let greeting = 42;
    greeting.to_string()
}
"#,
    );
}
#[test]
fn impl_spacing() {
    check(
        r#"script;

struct Foo {
    bar: u64,
    baz: bool,
}

trait Qux {
    fn is_baz_true(self) -> bool;
}

impl<A ,     B>    Qux<A, B> for
Foo
where
    A    : Qux,
    B: Qux    ,
{fn is_baz_true(self) -> bool {
        self.baz
    }}"#,
        r#"script;

struct Foo {
    bar: u64,
    baz: bool,
}

trait Qux {
    fn is_baz_true(self) -> bool;
}

impl<A, B> Qux<A, B> for Foo where
    A: Qux,
    B: Qux,
{
    fn is_baz_true(self) -> bool {
        self.baz
    }
}
"#,
    );
}

#[test]
fn impl_without_generics() {
    check(
        r#"script;

struct Foo {
    bar: u64,
    baz: bool,
}

trait Qux {
    fn is_baz_true(self) -> bool;
}

impl   Qux for
Foo
{fn is_baz_true(self) -> bool {
        self.baz
    }}"#,
        r#"script;

struct Foo {
    bar: u64,
    baz: bool,
}

trait Qux {
    fn is_baz_true(self) -> bool;
}

impl Qux for Foo {
    fn is_baz_true(self) -> bool {
        self.baz
    }
}
"#,
    );
}

#[test]
fn newline_sequence_formatting() {
    check(
        r#"script;

fn main() {
    let number: u64 = 10;

    let number2: u64 = 20;



    let number3: u64 = 30;



}"#,
        r#"script;

fn main() {
    let number: u64 = 10;

    let number2: u64 = 20;

    let number3: u64 = 30;
}
"#,
    );
}

#[test]
fn inner_doc_comments() {
    check(
        r#"script;

enum Color {
    //! Color is a Sway enum
    blue: (),
    red: ()
}

fn main() {
}"#,
        r#"script;

enum Color {
    //! Color is a Sway enum
    blue: (),
    red: (),
}

fn main() {}
"#,
    );
}

#[test]
fn outer_doc_comments() {
    check(
        r#"script;

enum Color {
    /// Blue color
    blue: (),
    /// Red color
    red: ()
}
/// This is the main function
fn main() {
}"#,
        r#"script;

enum Color {
    /// Blue color
    blue: (),
    /// Red color
    red: (),
}
/// This is the main function
fn main() {}
"#,
    );
}

#[test]
fn newline_comment_handler_interaction() {
    check(
        r#"script;

// use statements
use std::*;

fn main() {
    // Array of integers with type ascription
let array_of_integers: [u8;
5] = [1, 2, 3, 4, 5];


    // Array of strings
    let array_of_strings = [ "Bob", "Jan", "Ron"];
}
"#,
        r#"script;

// use statements
use std::*;

fn main() {
    // Array of integers with type ascription
    let array_of_integers: [u8; 5] = [1, 2, 3, 4, 5];

    // Array of strings
    let array_of_strings = ["Bob", "Jan", "Ron"];
}
"#,
    );
}
#[test]
fn comments_before_module_kind() {
    check(
        r#"// something about module kind
// something else about module kind
library;"#,
        r#"// something about module kind
// something else about module kind
library;
"#,
    );
}
#[test]
fn newline_before_comments() {
    check(
        r#"


// something about module kind
// something else about module kind
library;"#,
        r#"// something about module kind
// something else about module kind
library;
"#,
    );
}
#[test]
fn destructure_structs() {
    check(
        r#"library;

struct Point {
    x: u64,
    y: u64,
}
struct TupleInStruct {
    nested_tuple: (u64, (u32, (bool, str[2]))),
}
fn struct_destructuring() {
    let point1 = Point { x: 0, y: 0 };
    let Point{x, y} = point1;
    let point2 = Point { x: 18446744073709551615, y: 18446744073709551615};
    let Point{extremely_long_var_name, other_really_long_var_name} = point2;
    let tuple_in_struct = TupleInStruct {
        nested_tuple: (
            42u64,
            (42u32, (true, "ok"))
        ),
    };
    let TupleInStruct {
        nested_tuple: (a, (b, (c, d))),
    } = tuple_in_struct;
}
"#,
        r#"library;

struct Point {
    x: u64,
    y: u64,
}
struct TupleInStruct {
    nested_tuple: (u64, (u32, (bool, str[2]))),
}
fn struct_destructuring() {
    let point1 = Point { x: 0, y: 0 };
    let Point { x, y } = point1;
    let point2 = Point {
        x: 18446744073709551615,
        y: 18446744073709551615,
    };
    let Point {
        extremely_long_var_name,
        other_really_long_var_name,
    } = point2;
    let tuple_in_struct = TupleInStruct {
        nested_tuple: (42u64, (42u32, (true, "ok"))),
    };
    let TupleInStruct {
        nested_tuple: (a, (b, (c, d))),
    } = tuple_in_struct;
}
"#,
    );
}
#[test]
fn multiline_collections() {
    check(
        r#"library;
fn func_with_multiline_collections() {
    let x = (
        "hello",
        "world",
    );
}
"#,
        r#"library;
fn func_with_multiline_collections() {
    let x = ("hello", "world");
}
"#,
    );
}

#[test]
fn comments_between_if_else() {
    check(
        r#"script;

fn main() {
    if pledge_history_index != 0 {
        // This is a comment
        storage.pledge_history.insert((user, pledge_history_index), pledge);
    }
    // This is also a comment,
    // but multiline
    else if true {
        // This is yet another comment
        storage.pledge_count.insert(user, pledge_count + 1);
    }
    // This is the last comment
    else {
        storage.pledge_count.insert(user, pledge_count + 1);
    }
}
"#,
        r#"script;

fn main() {
    if pledge_history_index != 0 {
        // This is a comment
        storage.pledge_history.insert((user, pledge_history_index), pledge);
    }
    // This is also a comment,
    // but multiline
    else if true {
        // This is yet another comment
        storage.pledge_count.insert(user, pledge_count + 1);
    }
    // This is the last comment
    else {
        storage.pledge_count.insert(user, pledge_count + 1);
    }
}
"#,
    );
}
#[test]
fn parameterless_attributes() {
    check(
        r#"library;

abi MyContract {
    #[test]
    fn foo();
}
"#,
        r#"library;

abi MyContract {
    #[test]
    fn foo();
}
"#,
    );
}

#[test]
fn traits_with_def_block() {
    check(
        r#"script;

pub trait Foo {
    fn foo(self, other: Self);
} {
    fn bar(self, other: Self) {}
}

fn main() {}
"#,
        r#"script;

pub trait Foo {
    fn foo(self, other: Self);
} {
    fn bar(self, other: Self) {}
}

fn main() {}
"#,
    );
}

#[test]
fn if_else_multiline_to_inline() {
    check(
        r#"script;

fn main() {
    if foo    {
           let x = 1;
    } else    {
        bar(y)   ;
    }
}
"#,
        r#"script;

fn main() {
    if foo { let x = 1; } else { bar(y); }
}
"#,
    );
}

#[test]
fn if_else_retain_multiline() {
    check(
        r#"script;


fn main() {
    if foo    {
           let really_long_variable = 1;
    } else    {
        bar(y)   ;
    }
}
"#,
        r#"script;

fn main() {
    if foo {
        let really_long_variable = 1;
    } else {
        bar(y);
    }
}
"#,
    );
}

#[test]
fn multiple_comma_separated_attributes() {
    check(
        r#"library;

#[test, inline(always), storage(read, write), payable]
fn foo() {}
"#,
        r#"library;

#[test, inline(always), storage(read, write), payable]
fn foo() {}
"#,
    );
}

#[test]
fn stack_of_comma_separated_attributes1() {
    check(
        r#"library;

/// this is a doc comment
#[storage(read, write), payable]
#[test, inline(always)]
fn foo() {}
"#,
        r#"library;

/// this is a doc comment
#[storage(read, write), payable]
#[test, inline(always)]
fn foo() {}
"#,
    );
}

#[test]
fn stack_of_comma_separated_attributes2() {
    check(
        r#"library;

/// this is a doc comment
#[storage(read, write)]
#[payable]
#[test]
#[inline(always)]
fn foo() {}
"#,
        r#"library;

/// this is a doc comment
#[storage(read, write)]
#[payable]
#[test]
#[inline(always)]
fn foo() {}
"#,
    );
}

#[test]
fn comment_between_closing_brace_and_else() {
    check(
        r#"contract;

impl MyContract for Contract {
    fn is_blue() -> bool {
        if self == PrimaryColor::Blue {
            true
        }
            // Overindented comment, underindented else
    else if self == PrimaryColor::Red {
            true
        } // Trailing comment
    // Underindented comment
            // Overindented else
                else {
            false
        }
    }
}"#,
        r#"contract;

impl MyContract for Contract {
    fn is_blue() -> bool {
        if self == PrimaryColor::Blue {
            true
        }
        // Overindented comment, underindented else
        else if self == PrimaryColor::Red {
            true
        } // Trailing comment
        // Underindented comment
        // Overindented else
        else {
            false
        }
    }
}
"#,
    );
}

#[test]
fn comment_between_if_else_inline_to_multiline() {
    check(
        r#"contract;

impl MyContract for Contract {
    fn is_blue() -> bool {
            if self == PrimaryColor::Blue { true }
        // TODO remove this else when exhaustive ifs are checked for
    else { false }
    }
}"#,
        r#"contract;

impl MyContract for Contract {
    fn is_blue() -> bool {
        if self == PrimaryColor::Blue {
            true
        }
        // TODO remove this else when exhaustive ifs are checked for
        else {
            false
        }
    }
}
"#,
    );
}

#[test]
fn asm_block() {
    check(
        r#"library;

fn foo() {
    asm(r1: self, r2: other, r3, r4) {
        addi r3 zero i32;
        meq r4 r1 r2 r3;
        r4: bool
    }
}
"#,
        r#"library;

fn foo() {
    asm(r1: self, r2: other, r3, r4) {
        addi r3 zero i32;
        meq  r4 r1 r2 r3;
        r4: bool
    }
}
"#,
    );
}

#[test]
fn empty_blocks() {
    check(
        r#"contract;

fn contents() {
    let i = {    };
    match i {
    }
    if true {    }
}
fn empty() {}
"#,
        r#"contract;

fn contents() {
    let i = {};
    match i {}
    if true {}
}
fn empty() {}
"#,
    );
}

#[test]
fn abi_supertrait() {
    check(
        r#"contract;

trait ABIsupertrait {
    fn foo();
}

abi MyAbi : ABIsupertrait {
    fn bar();
} {
    fn baz() {
        Self::foo()     // supertrait method usage
    }
}

impl ABIsupertrait for Contract {
    fn foo() {}
}

// The implementation of MyAbi for Contract must also implement ABIsupertrait
impl MyAbi for Contract {
    fn bar() {
        Self::foo()     // supertrait method usage
    }
}
"#,
        r#"contract;

trait ABIsupertrait {
    fn foo();
}

abi MyAbi : ABIsupertrait {
    fn bar();
} {
    fn baz() {
        Self::foo() // supertrait method usage
    }
}

impl ABIsupertrait for Contract {
    fn foo() {}
}

// The implementation of MyAbi for Contract must also implement ABIsupertrait
impl MyAbi for Contract {
    fn bar() {
        Self::foo() // supertrait method usage
    }
}
"#,
    );
}

#[test]
fn test_comments_after_deps() {
    check(
        r#"library;

use std::{u256::U256, vec::*};
use ::utils::vec::sort;
use ::utils::numbers::*;

// pub fn aggregate_results(results: Vec<Vec<U256>>) -> Vec<U256> {
//     let mut aggregated = Vec::new();

//     let mut i = 0;
//     while (i < results.len) {
//         let values = results.get(i).unwrap();
//         aggregated.push(aggregate_values(values));

//         i += 1;
//     }

//     return aggregated;
// }"#,
        r#"library;

use std::{u256::U256, vec::*};
use ::utils::vec::sort;
use ::utils::numbers::*;

// pub fn aggregate_results(results: Vec<Vec<U256>>) -> Vec<U256> {
//     let mut aggregated = Vec::new();

//     let mut i = 0;
//     while (i < results.len) {
//         let values = results.get(i).unwrap();
//         aggregated.push(aggregate_values(values));

//         i += 1;
//     }

//     return aggregated;
// }
"#,
    );
}

#[test]
fn temporarily_commented_out_fn_with_doc_comments() {
    check(
        r#"contract;

abi MyContract {
    /// Doc comment
    /* 
        Some comment
    */
    fn test_function() -> bool;
}

impl MyContract for Contract {
    /// This is documentation for a commented out function
    // fn commented_out_function() {
    //}

    fn test_function() -> bool {
        true
    }
}"#,
        r#"contract;

abi MyContract {
    /// Doc comment
    /* 
        Some comment
    */
    fn test_function() -> bool;
}

impl MyContract for Contract {
    /// This is documentation for a commented out function
    // fn commented_out_function() {
    //}

    fn test_function() -> bool {
        true
    }
}
"#,
    );
}

#[test]
fn empty_impl() {
    check(
        r#"
library;

impl OrdEq for u256 {

}
        "#,
        r#"library;

impl OrdEq for u256 {}
"#,
    );
}

#[test]
fn empty_fn() {
    check(
        r#"
library;

fn test() {

}
        "#,
        r#"library;

fn test() {}
"#,
    );
}

#[test]
fn empty_if() {
    check(
        r#"
library;

fn test() {
    if ( something ( ) ) {

    }



}
        "#,
        r#"library;

fn test() {
    if (something()) {}
}
"#,
    );
}
