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
fn struct_alignment_with_public_fields() {
    let mut formatter = Formatter::default();
    formatter.config.structures.field_alignment = FieldAlignment::AlignFields(40);

    check_with_formatter(
        r#"contract;
pub struct Foo<T, P> {
   barbazfoo: u64,
   pub baz     : bool,
}
"#,
        r#"contract;
pub struct Foo<T, P> {
    barbazfoo : u64,
    pub baz   : bool,
}
"#,
        &mut formatter,
    );

    check_with_formatter(
        r#"contract;
pub struct Foo<T, P> {
   pub barbazfoo: u64,
   baz     : bool,
}
"#,
        r#"contract;
pub struct Foo<T, P> {
    pub barbazfoo : u64,
    baz           : bool,
}
"#,
        &mut formatter,
    );
}

#[test]
fn struct_public_fields() {
    let mut formatter = Formatter::default();
    formatter.config.structures.field_alignment = FieldAlignment::Off;

    check_with_formatter(
        r#"contract;
pub struct Foo<T, P> {
   pub  barbaz:   T,
   foo: u64,
     pub  baz  : bool,
}
"#,
        r#"contract;
pub struct Foo<T, P> {
    pub barbaz: T,
    foo: u64,
    pub baz: bool,
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
              ns1 { var3: u64 = 1, ns2 { var4: u64 = 1 } },
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
    ns1 {
        var3: u64 = 1,
        ns2 {
            var4: u64 = 1,
        },
    },
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
      ns1 { var3: u64 = 1,  ns2 { var4: u64 = 1, },  }, var5: u64 = 1
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
    ns1 {
        var3      : u64 = 1,
        ns2 {
            var4  : u64 = 1,
        },
    },
    var5          : u64 = 1
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
    T: String,
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

    fuel_coin.force_transfer {
        gas: default_gas,
    }(3, fuelcoin_id, balance_test_id);

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

impl<A, B> Qux<A, B> for Foo
where
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
        storage
            .pledge_history
            .insert((user, pledge_history_index), pledge);
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
        meq r4 r1 r2 r3;
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

#[test]
fn bug_whitespace_added_after_comment() {
    check(
        r#"
library;
// GTF Opcode const selectors
//
pub const GTF_OUTPUT_TYPE = 0x300;
pub const GTF_OUTPUT_COIN_TO = 0x301;
pub const GTF_OUTPUT_COIN_AMOUNT = 0x302;
pub const GTF_OUTPUT_COIN_ASSET_ID = 0x303;
// pub const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x304;
// pub const GTF_OUTPUT_CONTRACT_BALANCE_ROOT = 0x305;
// pub const GTF_OUTPUT_CONTRACT_STATE_ROOT = 0x306;
// pub const GTF_OUTPUT_CONTRACT_CREATED_CONTRACT_ID = 0x307;
// pub const GTF_OUTPUT_CONTRACT_CREATED_STATE_ROOT = 0x308;



/// The output type for a transaction.
pub enum Output {
    /// A coin output.
    Coin: (), /// A contract output.
    Contract: (),
    /// Remaining "change" from spending of a coin.
    Change: (),
        /// A variable output.
    Variable: (),
}
    /// The output type for a transaction.
pub enum Input {
        /// A variable output.
            Variable: (),
}
        "#,
        r#"library;
// GTF Opcode const selectors
//
pub const GTF_OUTPUT_TYPE = 0x300;
pub const GTF_OUTPUT_COIN_TO = 0x301;
pub const GTF_OUTPUT_COIN_AMOUNT = 0x302;
pub const GTF_OUTPUT_COIN_ASSET_ID = 0x303;
// pub const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x304;
// pub const GTF_OUTPUT_CONTRACT_BALANCE_ROOT = 0x305;
// pub const GTF_OUTPUT_CONTRACT_STATE_ROOT = 0x306;
// pub const GTF_OUTPUT_CONTRACT_CREATED_CONTRACT_ID = 0x307;
// pub const GTF_OUTPUT_CONTRACT_CREATED_STATE_ROOT = 0x308;

/// The output type for a transaction.
pub enum Output {
    /// A coin output.
    Coin: (),
    /// A contract output.
    Contract: (),
    /// Remaining "change" from spending of a coin.
    Change: (),
    /// A variable output.
    Variable: (),
}
/// The output type for a transaction.
pub enum Input {
    /// A variable output.
    Variable: (),
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
fn chained_methods_0() {
    check(
        r#"
library;

fn test() {
    fuel.really_long_field.other_really_long_field.foo().bar().baz.quux().yet_another_call().to_go_above_max_line_length();
}
        "#,
        r#"library;

fn test() {
    fuel.really_long_field
        .other_really_long_field
        .foo()
        .bar()
        .baz
        .quux()
        .yet_another_call()
        .to_go_above_max_line_length();
}
"#,
    );
}

#[test]
fn chained_methods_1() {
    check(
        r#"
library;

fn test() {
    fuelcoin.really_long_field.other_really_long_field.foo().bar().baz.quux().yet_another_call().to_go_above_max_line_length();
}
        "#,
        r#"library;

fn test() {
    fuelcoin
        .really_long_field
        .other_really_long_field
        .foo()
        .bar()
        .baz
        .quux()
        .yet_another_call()
        .to_go_above_max_line_length();
}
"#,
    );
}

#[test]
fn chained_methods_2() {
    check(
        r#"
library;

fn test() {
    fuelcoin.really_long_field.other_really_long_field.foo().bar().baz.quux([1,2]).yet_another_call([1, 2, 3, 4, 6, 7, 7, 8, 8, 9, 9, 9, 19, 1123123, 12312, 312, 312, 3123, 12,31, 44],[1,2], true).to_go_above_max_line_length();ping();
}
        "#,
        r#"library;

fn test() {
    fuelcoin
        .really_long_field
        .other_really_long_field
        .foo()
        .bar()
        .baz
        .quux([1, 2])
        .yet_another_call(
            [
                1, 2, 3, 4, 6, 7, 7, 8, 8, 9, 9, 9, 19, 1123123, 12312, 312, 312,
                3123, 12, 31, 44,
            ],
            [1, 2],
            true,
        )
        .to_go_above_max_line_length();
    ping();
}
"#,
    );
}

#[test]
fn chained_methods_3() {
    check(
        r#"
library;

fn test() {
    fuelcoin.really_long_field.other_really_long_field.foo().bar().baz.quux([1,2]).yet_another_call(1, 2, 3, 4, 6, 7, 7, 8, 8, 9, 9, 9, 19, 1123123, 12312, 312, 312, 3123, 12,31, 44,[1,2], true).to_go_above_max_line_length();
}
        "#,
        r#"library;

fn test() {
    fuelcoin
        .really_long_field
        .other_really_long_field
        .foo()
        .bar()
        .baz
        .quux([1, 2])
        .yet_another_call(
            1,
            2,
            3,
            4,
            6,
            7,
            7,
            8,
            8,
            9,
            9,
            9,
            19,
            1123123,
            12312,
            312,
            312,
            3123,
            12,
            31,
            44,
            [1, 2],
            true,
        )
        .to_go_above_max_line_length();
}
"#,
    );
}

#[test]
fn comment_in_the_middle() {
    check(
        r#"
        library;

        fn test() {
            let number: /* this number is for counting */ u64 = 10;



        }"#,
        r#"library;

fn test() {
    let number: /* this number is for counting */ u64 = 10;
}
"#,
    );
}

#[test]
fn trait_multiline_method_x() {
    check(
        r#"library; trait MyComplexTrait { fn complex_function( arg1: MyStruct<[b256; 3], u8>, arg2: [MyStruct<u64, bool>; 4],
        arg3: (str[5], bool),
        arg4: MyOtherStruct) -> str[6]; }"#,
        r#"library;
trait MyComplexTrait {
    fn complex_function(
        arg1: MyStruct<[b256; 3], u8>,
        arg2: [MyStruct<u64, bool>; 4],
        arg3: (str[5], bool),
        arg4: MyOtherStruct,
    ) -> str[6];
}
"#,
    );
}

#[test]
fn long_array() {
    check(
        r#"library;
        fn main() {
            let x = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,99,22];
        }"#,
        r#"library;
fn main() {
    let x = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
        22, 99, 22,
    ];
}
"#,
    );
}

#[test]
fn struct_new_line() {
    check(
        r#"
        library;

struct Item {
    price: u64, amount: u64,
    id: u64,
}

enum MyEnum {
    Item: Item,
}

fn main() {
    let my_enum = MyEnum::Item(Item {
        price: 5, amount: 2, id: 42,
    });
    my_enum.test(Item {
        price: 5, amount: 2, id: 42,
    });
}
            "#,
        r#"library;

struct Item {
    price: u64,
    amount: u64,
    id: u64,
}

enum MyEnum {
    Item: Item,
}

fn main() {
    let my_enum = MyEnum::Item(Item {
        price: 5,
        amount: 2,
        id: 42,
    });
    my_enum.test(Item {
        price: 5,
        amount: 2,
        id: 42,
    });
}
"#,
    )
}

#[test]
fn struct_new_line_2() {
    check(
        r#"
        library;
impl Convert<Square> for Rectangle { fn from(t: Square) -> Self {
Self { width: t
    .width,length: t.width,
}}}
            "#,
        r#"library;
impl Convert<Square> for Rectangle {
    fn from(t: Square) -> Self {
        Self {
            width: t.width,
            length: t.width,
        }
    }
}
"#,
    )
}

#[test]
fn func_call_long_args() {
    check(
        r#"
        library;

        fn access_control_with_identity() {
            // ANCHOR: access_control_with_identity
            let sender = msg_sender().unwrap();
            require(sender == storage.owner.read(), MyError::UnauthorizedUser(sender));
            // ANCHOR_END: access_control_with_identity
        }
    "#,
        r#"library;

fn access_control_with_identity() {
    // ANCHOR: access_control_with_identity
    let sender = msg_sender().unwrap();
    require(
        sender == storage
            .owner
            .read(),
        MyError::UnauthorizedUser(sender),
    );
    // ANCHOR_END: access_control_with_identity
}
"#,
    )
}

#[test]
fn func_call_long_args_with_long_expr() {
    check(
        r#"
        library;

        fn access_control_with_identity() {
            // ANCHOR: access_control_with_identity
            let sender = msg_sender().unwrap();
            require(sender == storage.owner.read().some_prop().that_is_too_long_to_fit_in_one_line().or_two_lines(), MyError::UnauthorizedUser(sender));
            // ANCHOR_END: access_control_with_identity
        }
    "#,
        r#"library;

fn access_control_with_identity() {
    // ANCHOR: access_control_with_identity
    let sender = msg_sender().unwrap();
    require(
        sender == storage
            .owner
            .read()
            .some_prop()
            .that_is_too_long_to_fit_in_one_line()
            .or_two_lines(),
        MyError::UnauthorizedUser(sender),
    );
    // ANCHOR_END: access_control_with_identity
}
"#,
    )
}

#[test]
fn method_call_long_args_with_long_expr() {
    check(
        r#"
        library;

        fn access_control_with_identity() {
            // ANCHOR: access_control_with_identity
            let sender = msg_sender().unwrap();
            sender.require(sender == storage.owner.read().some_prop().that_is_too_long_to_fit_in_one_line().or_two_lines(), MyError::UnauthorizedUser(sender));
            // ANCHOR_END: access_control_with_identity
        }
    "#,
        r#"library;

fn access_control_with_identity() {
    // ANCHOR: access_control_with_identity
    let sender = msg_sender().unwrap();
    sender.require(
        sender == storage
            .owner
            .read()
            .some_prop()
            .that_is_too_long_to_fit_in_one_line()
            .or_two_lines(),
        MyError::UnauthorizedUser(sender),
    );
    // ANCHOR_END: access_control_with_identity
}
"#,
    )
}

#[test]
fn while_too_long_expr() {
    check(
        r#"
    library;

    fn main() {
    // ANCHOR_END: vec_get
    // ANCHOR: vec_get_oob
    let does_not_exist = v.get(100);
    // ...decide here how to handle an out-of-bounds access
    // ANCHOR_END: vec_get_oob
    // ANCHOR: vec_iterate
    let mut i = 0;
    while i < v.len() {
        log(v.get(i).unwrap());
        i += 1;
    }
    // ANCHOR_END: vec_iterate
    // ANCHOR: vec_multiple_data_types
    enum TableCell {
        Int: u64,
        B256: b256,
        Boolean: bool,
    }
}
"#,
        r#"library;

fn main() {
    // ANCHOR_END: vec_get
    // ANCHOR: vec_get_oob
    let does_not_exist = v.get(100);
    // ...decide here how to handle an out-of-bounds access
    // ANCHOR_END: vec_get_oob
    // ANCHOR: vec_iterate
    let mut i = 0;
    while i < v.len() {
        log(v.get(i).unwrap());
        i += 1;
    }
    // ANCHOR_END: vec_iterate
    // ANCHOR: vec_multiple_data_types
    enum TableCell {
        Int: u64,
        B256: b256,
        Boolean: bool,
    }
}
"#,
    );
}

#[test]
fn match_as_arg() {
    check(
        r#"
    library;

    fn main() {
         assert(match color { Color::Blue => true,
    _ => false,
    });
}
"#,
        r#"library;

fn main() {
    assert(match color {
        Color::Blue => true,
        _ => false,
    });
}
"#,
    );
}

#[test]
fn single_long_arg() {
    check(
        r#"library;

    fn main() {
        if foo {
            // ANCHOR: storage_map_insert
                let addr1 = Address::from(0x010101010101010101010101010101010101010101010101010101010101010101010101010101);
            }
    }
    "#,
        r#"library;

fn main() {
    if foo {
        // ANCHOR: storage_map_insert
        let addr1 = Address::from(
            0x010101010101010101010101010101010101010101010101010101010101010101010101010101,
        );
    }
}
"#,
    );
}

#[test]
fn method_call_2() {
    check(
        r#"
    library;

fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(Wallet, contract_address);
    let amount_to_send = 200;
    let recipient_address = Address::from(contract_address);
    caller.send_funds { gas: 10000, coins: 0, asset_id: ZERO_B256 }(
            amount_to_send,
            recipient_address,
        );
}

    "#,
        r#"library;

fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(Wallet, contract_address);
    let amount_to_send = 200;
    let recipient_address = Address::from(contract_address);
    caller
        .send_funds {
            gas: 10000,
            coins: 0,
            asset_id: ZERO_B256,
        }(amount_to_send, recipient_address);
}
"#,
    )
}

#[test]
fn method_call_3() {
    check(
        r#"library;

fn enum_in_vec() -> Vec<Pasta> {
   let mut vec: Vec<Pasta> = Vec::new();
   vec.push(Pasta::Tortelini(Bimba {
      bim: 1111,
      bam: 2222_u32,
   }));
   vec.push(Pasta::Rigatoni(1987));
   vec.push(Pasta::Spaghetti(true));
   vec
}
       "#,
        r#"library;

fn enum_in_vec() -> Vec<Pasta> {
    let mut vec: Vec<Pasta> = Vec::new();
    vec.push(Pasta::Tortelini(Bimba {
        bim: 1111,
        bam: 2222_u32,
    }));
    vec.push(Pasta::Rigatoni(1987));
    vec.push(Pasta::Spaghetti(true));
    vec
}
"#,
    );
}

#[test]
fn test_comment_random_places() {
    check(
        r#"library;

fn enum_in_vec() -> Vec<Pasta> {
    let number: /*
        this number is for counting
    */ u64 = 10;
    let number: // this number is for counting
    u64 = 10;
}
    "#,
        r#"library;

fn enum_in_vec() -> Vec<Pasta> {
    let number: /*
        this number is for counting
    */ u64 = 10;
    let number: // this number is for counting
 u64 = 10;
}
"#,
    );
}

#[test]
fn test_comment_v2() {
    check(
        r#"library;
    /// This is documentation for a commented out function
    // fn commented_out_function() {
    //}
        
    fn test_function() -> bool {
        true
    }
        "#,
        r#"library;
/// This is documentation for a commented out function
// fn commented_out_function() {
//}

fn test_function() -> bool {
    true
}
"#,
    );
}

#[test]
fn long_doc_break_new_line() {
    check(
        r#"library;

/// Allocates zeroed memory on the heap.
///
/// # Additional Information
///
/// In the FuelVM, the heap begins at `VM_MAX_RAM` and grows downward.
/// The heap pointer(`$hp`) always points to the first allocated byte.
///
/// Initially the heap will look like this:
/// ```
///                                                     ▾$hp
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                                                     ▴VM_MAX_RAM
/// ```
/// After allocating with `let ptr = alloc::<u64>(1)`:
/// ```
///                             ▾$hp
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                             ▴ptr                    ▴VM_MAX_RAM
/// ```
/// After writing with `sw(ptr, u64::max())`:
/// ```
///                             ▾$hp
/// ... 00 00 00 00 00 00 00 00 FF FF FF FF FF FF FF FF |
///                             ▴ptr                    ▴VM_MAX_RAM
/// ```
/// For more information, see the Fuel Spec for [VM Initialization](https://fuellabs.github.io/fuel-specs/master/vm#vm-initialization)
/// and the VM Instruction Set for [Memory Allocation](https://docs.fuel.network/docs/specs/fuel-vm/instruction-set#aloc-allocate-memory).
///
/// # Arguments
/// 
/// * `count`: [u64] - The number of `size_of<T>` bytes to allocate onto the heap.
///
/// # Returns
///
/// * [raw_ptr] - The pointer to the newly allocated memory.
///
/// # Examples
///
/// ```sway
/// use std::alloc::alloc;
/// 
/// fn foo() {
///     let ptr = alloc::<u64>(2);
///     assert(!ptr.is_null());
/// }
/// ```
/// Reallocates the given area of memory.
/// 
/// # Arguments
///
/// * `ptr`: [raw_ptr] - The pointer to the area of memory to reallocate.
/// * `count`: [u64] - The number of `size_of<T>` bytes kept when reallocating. These are not set to 0.
/// * `new_count`: [u64] - The number of new `size_of<T>` bytes to allocate. These are set to 0.
///
/// # Returns
/// 
/// * [raw_ptr] - The pointer to the newly reallocated memory.
///
/// # Examples
///
/// ```sway
/// use std::alloc::{alloc, realloc};
///
/// fn foo() {
///     let ptr = alloc::<u64>(1);
///     ptr.write(5);
///     let reallocated_ptr = realloc::<u64>(ptr, 1, 2);
///     assert(reallocated_ptr.read::<u64>() == 5);
/// }
/// ```
pub fn realloc<T>(ptr: raw_ptr, count: u64, new_count: u64) -> raw_ptr {
    if new_count > count {
        let new_ptr = alloc::<T>(new_count);
        if count > 0 {
            ptr.copy_to::<T>(new_ptr, count);
        }
        new_ptr
    } else {
        ptr
    }
}

/// Allocates zeroed memory on the heap in individual bytes.
pub fn alloc_bytes(count: u64) -> raw_ptr {
    asm(size: count, ptr) {
        aloc size;
        move ptr hp;
        ptr: raw_ptr
    }
}

        "#,
        r#"library;

/// Allocates zeroed memory on the heap.
///
/// # Additional Information
///
/// In the FuelVM, the heap begins at `VM_MAX_RAM` and grows downward.
/// The heap pointer(`$hp`) always points to the first allocated byte.
///
/// Initially the heap will look like this:
/// ```
///                                                     ▾$hp
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                                                     ▴VM_MAX_RAM
/// ```
/// After allocating with `let ptr = alloc::<u64>(1)`:
/// ```
///                             ▾$hp
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                             ▴ptr                    ▴VM_MAX_RAM
/// ```
/// After writing with `sw(ptr, u64::max())`:
/// ```
///                             ▾$hp
/// ... 00 00 00 00 00 00 00 00 FF FF FF FF FF FF FF FF |
///                             ▴ptr                    ▴VM_MAX_RAM
/// ```
/// For more information, see the Fuel Spec for [VM Initialization](https://fuellabs.github.io/fuel-specs/master/vm#vm-initialization)
/// and the VM Instruction Set for [Memory Allocation](https://docs.fuel.network/docs/specs/fuel-vm/instruction-set#aloc-allocate-memory).
///
/// # Arguments
///
/// * `count`: [u64] - The number of `size_of<T>` bytes to allocate onto the heap.
///
/// # Returns
///
/// * [raw_ptr] - The pointer to the newly allocated memory.
///
/// # Examples
///
/// ```sway
/// use std::alloc::alloc;
///
/// fn foo() {
///     let ptr = alloc::<u64>(2);
///     assert(!ptr.is_null());
/// }
/// ```
/// Reallocates the given area of memory.
///
/// # Arguments
///
/// * `ptr`: [raw_ptr] - The pointer to the area of memory to reallocate.
/// * `count`: [u64] - The number of `size_of<T>` bytes kept when reallocating. These are not set to 0.
/// * `new_count`: [u64] - The number of new `size_of<T>` bytes to allocate. These are set to 0.
///
/// # Returns
///
/// * [raw_ptr] - The pointer to the newly reallocated memory.
///
/// # Examples
///
/// ```sway
/// use std::alloc::{alloc, realloc};
///
/// fn foo() {
///     let ptr = alloc::<u64>(1);
///     ptr.write(5);
///     let reallocated_ptr = realloc::<u64>(ptr, 1, 2);
///     assert(reallocated_ptr.read::<u64>() == 5);
/// }
/// ```
pub fn realloc<T>(ptr: raw_ptr, count: u64, new_count: u64) -> raw_ptr {
    if new_count > count {
        let new_ptr = alloc::<T>(new_count);
        if count > 0 {
            ptr.copy_to::<T>(new_ptr, count);
        }
        new_ptr
    } else {
        ptr
    }
}

/// Allocates zeroed memory on the heap in individual bytes.
pub fn alloc_bytes(count: u64) -> raw_ptr {
    asm(size: count, ptr) {
        aloc size;
        move ptr hp;
        ptr: raw_ptr
    }
}
"#,
    )
}

#[test]
fn broken_doc_comment() {
    check(
        r#"library;
        /// line 1
        /// line 2
        // line 3
        /// line 4
        // line 5
        /// line 6
        fn test() {
        }
    "#,
        r#"library;
/// line 1
/// line 2
// line 3
/// line 4
// line 5
/// line 6
fn test() {}
"#,
    );
}

#[test]
fn asm_block_v2() {
    check(
        r#"library;
    
    pub fn transfer(self, asset_id: AssetId, amount: u64) {
        // maintain a manual index as we only have `while` loops in sway atm:
        let mut index = 0;

        asm(input: input) {
            input: u256
        }

        // If an output of type `OutputVariable` is found, check if its `amount` is
        // zero. As one cannot transfer zero coins to an output without a panic, a
        // variable output with a value of zero is by definition unused.
        let number_of_outputs = output_count();
        while index < number_of_outputs {
            if let Output::Variable = output_type(index) {
                if output_amount(index) == 0 {
                    asm(r1: self.value, r2: index, r3: amount, r4: asset_id.value.with_long_long_name) {
                        tro r1 r2 r3 r4;
                    };
                    return;
                }
            }
            index += 1;
        }

        revert(FAILED_TRANSFER_TO_ADDRESS_SIGNAL);
    }
    "#,
        r#"library;

pub fn transfer(self, asset_id: AssetId, amount: u64) {
    // maintain a manual index as we only have `while` loops in sway atm:
    let mut index = 0;

    asm(input: input) {
        input: u256
    }

    // If an output of type `OutputVariable` is found, check if its `amount` is
    // zero. As one cannot transfer zero coins to an output without a panic, a
    // variable output with a value of zero is by definition unused.
    let number_of_outputs = output_count();
    while index < number_of_outputs {
        if let Output::Variable = output_type(index) {
            if output_amount(index) == 0 {
                asm(
                    r1: self.value,
                    r2: index,
                    r3: amount,
                    r4: asset_id.value.with_long_long_name,
                ) {
                    tro r1 r2 r3 r4;
                };
                return;
            }
        }
        index += 1;
    }

    revert(FAILED_TRANSFER_TO_ADDRESS_SIGNAL);
}
"#,
    );
}

#[test]
fn long_expr_assign() {
    check(
        r#"library;

fn foo() {
    let x = self.a > other.a || (self.a == other.a && (self.b > other.b || (self.b == other.b && (self.c > other.c || (self.c == other.c && self.d > other.d)))));
}
    "#,
        r#"library;

fn foo() {
    let x = self.a > other.a
        || (self.a == other.a
            && (self.b > other.b
                || (self.b == other.b
                    && (self.c > other.c
                        || (self.c == other.c
                            && self.d > other.d)))));
}
"#,
    );
}

#[test]
fn long_expr_return() {
    check(
        r#"library;

fn foo() {
    self.a > other.a || (self.a == other.a && (self.b > other.b || (self.b == other.b && (self.c > other.c || (self.c == other.c && self.d > other.d)))));
}
    "#,
        r#"library;

fn foo() {
    self.a > other.a
        || (self.a == other.a
            && (self.b > other.b
                || (self.b == other.b
                    && (self.c > other.c
                        || (self.c == other.c
                            && self.d > other.d)))));
}
"#,
    );
}

#[test]
fn long_expr_single_arg() {
    check(
        r#"library;
fn test() {
    Self::from((
        self.b * other.c + result_b_d.upper + overflow_of_b_to_a_3 + overflow_of_b_to_a_2 + overflow_of_b_to_a_1 + overflow_of_b_to_a_0,
        b,
        c,
        result_d_d.lower,
    ))
}
    "#,
        r#"library;
fn test() {
    Self::from((
        self.b * other.c + result_b_d.upper + overflow_of_b_to_a_3 + overflow_of_b_to_a_2 + overflow_of_b_to_a_1 + overflow_of_b_to_a_0,
        b,
        c,
        result_d_d.lower,
    ))
}
"#,
    );
}

#[test]
fn single_argument_not() {
    check(
        r#"library;
fn test() {
    assert(!(U256::from((0, 0, 0, 1)) > U256::from((0, u64::max(), 0, 0))));
}
    "#,
        r#"library;
fn test() {
    assert(!(U256::from((0, 0, 0, 1)) > U256::from((0, u64::max(), 0, 0))));
}
"#,
    );
}

#[test]
fn use_sorting_items() {
    check(
        r#"library;
    
    use ::option::Option::{*, self, z, foo, bar};
"#,
        r#"library;

use ::option::Option::{self, bar, foo, z, *};
"#,
    );
}

#[test]
fn whitespace_after_doccomment() {
    check(
        r#"library;
    
/// Trait to evaluate if one value is greater than or equal, or less than or equal to another of the same type.
trait OrdEq: Ord + Eq {
} {
    /// Evaluates if one value of the same type is greater or equal to than another.
    ///
    /// # Additional Information
    ///
    /// This trait requires that the `Ord` and `Eq` traits are implemented.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if `self` is greater than or equal to `other`, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Eq for MyStruct {
    ///     fn eq(self, other: Self) -> bool {
    ///         self.val == other.val
    ///     }
    /// }
    ///
    /// impl Ord for MyStruct {
    ///     fn gt(self, other: Self) -> bool {
    ///         self.val > other.val
    ///     }
    /// }
    ///
    /// impl OrdEq for MyStruct {}
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 10 };
    ///     let result = struct1 >= struct2;
    ///     assert(result);
    /// }
    /// ```
    fn ge(self, other: Self) -> bool {
        self.gt(other) || self.eq(other)
    }
    /// Some test
    fn test() {

    }
}
    "#,
        r#"library;

/// Trait to evaluate if one value is greater than or equal, or less than or equal to another of the same type.
trait OrdEq: Ord + Eq {
} {
    /// Evaluates if one value of the same type is greater or equal to than another.
    ///
    /// # Additional Information
    ///
    /// This trait requires that the `Ord` and `Eq` traits are implemented.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if `self` is greater than or equal to `other`, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Eq for MyStruct {
    ///     fn eq(self, other: Self) -> bool {
    ///         self.val == other.val
    ///     }
    /// }
    ///
    /// impl Ord for MyStruct {
    ///     fn gt(self, other: Self) -> bool {
    ///         self.val > other.val
    ///     }
    /// }
    ///
    /// impl OrdEq for MyStruct {}
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 10 };
    ///     let result = struct1 >= struct2;
    ///     assert(result);
    /// }
    /// ```
    fn ge(self, other: Self) -> bool {
        self.gt(other) || self.eq(other)
    }
    /// Some test
    fn test() {}
}
"#,
    );
}

#[test]
fn single_argument_method() {
    check(
        r#"library;
    
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let a = u64::from_be_bytes(
            [
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4],
                bytes[5], bytes[6], bytes[7],
            ],
        );
    }
    "#,
        r#"library;

pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
    let a = u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
}
"#,
    );
}

#[test]
fn impl_func_where() {
    check(
        r#"library;

        impl<K, V> Foo<Bar<K, V>>
        where
            K: Hash,
        {
            pub fn baz(self, _: K, _: V)
            where
                K: Hash,
        {
                debug();
            }
        }
    "#,
        r#"library;

impl<K, V> Foo<Bar<K, V>>
where
    K: Hash,
{
    pub fn baz(self, _: K, _: V)
    where
        K: Hash,
    {
        debug();
    }
}
"#,
    );
}
