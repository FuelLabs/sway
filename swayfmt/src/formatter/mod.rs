use self::shape::Shape;
use crate::parse::parse_file;
use crate::utils::map::{
    comments::handle_comments, newline::handle_newlines, newline_style::apply_newline_style,
};
pub use crate::{
    config::manifest::Config,
    error::{ConfigError, FormatterError},
};
use std::{fmt::Write, path::Path, sync::Arc};
use sway_core::BuildConfig;

pub(crate) mod shape;

#[derive(Debug, Default, Clone)]
pub struct Formatter {
    pub shape: Shape,
    pub config: Config,
}

pub type FormattedCode = String;

pub trait Format {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError>;
}

impl Formatter {
    pub fn from_dir(dir: &Path) -> Result<Self, ConfigError> {
        let config = match Config::from_dir(dir) {
            Ok(config) => config,
            Err(ConfigError::NotFound) => Config::default(),
            Err(e) => return Err(e),
        };

        Ok(Self {
            config,
            ..Default::default()
        })
    }
    pub fn format(
        &mut self,
        src: Arc<str>,
        build_config: Option<&BuildConfig>,
    ) -> Result<FormattedCode, FormatterError> {
        // apply the width heuristics settings from the `Config`
        self.shape.apply_width_heuristics(
            self.config
                .heuristics
                .heuristics_pref
                .to_width_heuristics(self.config.whitespace.max_width),
        );
        let src = src.trim();

        let path = build_config.map(|build_config| build_config.canonical_root_module());
        // Formatted code will be pushed here with raw newline stlye.
        // Which means newlines are not converted into system-specific versions until `apply_newline_style()`.
        // Use the length of src as a hint of the memory size needed for `raw_formatted_code`,
        // which will reduce the number of reallocations
        let mut raw_formatted_code = String::with_capacity(src.len());

        let module = parse_file(Arc::from(src), path.clone())?;
        module.format(&mut raw_formatted_code, self)?;

        let mut formatted_code = String::from(&raw_formatted_code);

        // Add comments
        handle_comments(
            Arc::from(src),
            &module,
            Arc::from(formatted_code.clone()),
            path.clone(),
            &mut formatted_code,
        )?;
        // Add newline sequences
        handle_newlines(
            Arc::from(src),
            &module,
            Arc::from(formatted_code.clone()),
            path,
            &mut formatted_code,
            self,
        )?;
        // Replace newlines with specified `NewlineStyle`
        apply_newline_style(
            self.config.whitespace.newline_style,
            &mut formatted_code,
            &raw_formatted_code,
        )?;
        if !formatted_code.ends_with('\n') {
            writeln!(formatted_code)?;
        }

        Ok(formatted_code)
    }
    pub(crate) fn with_shape<F, O>(&mut self, new_shape: Shape, f: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        let prev_shape = self.shape;
        self.shape = new_shape;
        let output = f(self);
        self.shape = prev_shape;

        output // used to extract an output if needed
    }
}

#[cfg(test)]
mod tests {
    use super::Formatter;
    use crate::config::user_def::FieldAlignment;
    use std::sync::Arc;

    /// Checks if the formatter is producing the same output when given it's output.
    fn test_stability(formatted_input: String, formatter: Formatter) -> bool {
        let mut formatter = formatter;
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(formatted_input.clone()), None).unwrap();
        formatted_input == formatted_sway_code
    }

    #[test]
    fn test_const() {
        let sway_code_to_format = r#"contract;
pub const TEST:u16=10;"#;
        let correct_sway_code = r#"contract;
pub const TEST: u16 = 10;
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_struct_alignment() {
        let sway_code_to_format = r#"contract;
pub struct Foo<T, P> {
   barbazfoo: u64,
   baz  : bool,
}
"#;
        let correct_sway_code = r#"contract;
pub struct Foo<T, P> {
    barbazfoo : u64,
    baz       : bool,
}
"#;

        let mut formatter = Formatter::default();
        formatter.config.structures.field_alignment = FieldAlignment::AlignFields(40);
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_struct() {
        let sway_code_to_format = r#"contract;
pub struct Foo {
    bar: u64,
    baz: bool
}
"#;
        let correct_sway_code = r#"contract;
pub struct Foo {
    bar: u64,
    baz: bool,
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_enum_without_variant_alignment() {
        let sway_code_to_format = r#"contract;

enum Color {
    Blue: (), Green: (),
            Red: (),
    Silver: (),
                    Grey: () }
        "#;
        let correct_sway_code = r#"contract;

enum Color {
    Blue: (),
    Green: (),
    Red: (),
    Silver: (),
    Grey: (),
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_enum_with_variant_alignment() {
        let sway_code_to_format = r#"contract;

enum Color {
    Blue: (), Green: (),
            Red: (),
    Silver: (),
                    Grey: (), }
        "#;
        let correct_sway_code = r#"contract;

enum Color {
    Blue   : (),
    Green  : (),
    Red    : (),
    Silver : (),
    Grey   : (),
}
"#;

        // Creating a config with enum_variant_align_threshold that exceeds longest variant length
        let mut formatter = Formatter::default();
        formatter.config.structures.field_alignment = FieldAlignment::AlignFields(20);

        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_item_abi_with_generics_and_attributes() {
        let sway_code_to_format = r#"contract;

abi StorageMapExample {
    #[storage(write)]fn insert_into_map1(key: u64, value: u64);

fn hello(key: u64, value: u64);
}"#;
        let correct_sway_code = r#"contract;

abi StorageMapExample {
    #[storage(write)]
    fn insert_into_map1(key: u64, value: u64);

    fn hello(key: u64, value: u64);
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_multi_items() {
        let sway_code_to_format = r#"contract;

pub const TEST: u16 = 10;
pub const TEST1: u16 = 10;"#;
        let correct_sway_code = r#"contract;

pub const TEST: u16 = 10;
pub const TEST1: u16 = 10;
"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_ty_formatting() {
        let sway_code_to_format = r#"contract;

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
    type,
    TupleNil: (),
    Tuple: (   u64,
        u32
    ),
}"#;
        let correct_sway_code = r#"contract;

enum TestTy {
    Infer: _,
    Array: [u8; 40],
    String: str[4],
    PathType: root::example::type,
    TupleNil: (),
    Tuple: (u64, u32),
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_storage_without_alignment() {
        let sway_code_to_format = r#"contract;
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
        "#;
        let correct_sway_code = r#"contract;
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
"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_storage_with_alignment() {
        let sway_code_to_format = r#"contract;
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
"#;
        let correct_sway_code = r#"contract;
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
"#;

        let mut formatter = Formatter::default();
        formatter.config.structures.field_alignment = FieldAlignment::AlignFields(50);
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_storage_initializer() {
        let sway_code_to_format = r#"contract;

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
}"#;
        let correct_sway_code = r#"contract;

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
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_item_fn() {
        let sway_code_to_format = r#"contract;

pub fn hello( person: String ) -> String {let greeting = 42;greeting.to_string()}
fn goodbye() -> usize {let farewell: usize = 5; farewell }"#;
        let correct_sway_code = r#"contract;

pub fn hello(person: String) -> String {
    let greeting = 42;
    greeting.to_string()
}
fn goodbye() -> usize {
    let farewell: usize = 5;
    farewell
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_same_line_where() {
        let sway_code_to_format = r#"contract;

pub fn hello( person: String ) -> String where T: Eq,{let greeting = 42;greeting.to_string()}"#;
        let correct_sway_code = r#"contract;

pub fn hello(person: String) -> String
where
    T: Eq,
{
    let greeting = 42;
    greeting.to_string()
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_trait_and_super_trait() {
        let sway_code_to_format = r#"library traits;

trait Person{ fn name( self )->String;fn age( self )->usize; }
trait Student:Person {fn university(self) -> String;}
trait Programmer {fn fav_language(self) -> String;}
trait CompSciStudent: Programmer+Student {fn git_username(self) -> String;}
trait TraitWithGenerics<T> where T: String {fn from(b: T) -> Self;}"#;
        let correct_sway_code = r#"library traits;

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
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_method_calls() {
        let sway_code_to_format = r#"script;

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
}"#;

        let correct_sway_code = r#"script;

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
"#;
        let mut formatter = Formatter::default();
        formatter.config.structures.small_structures_single_line = true;
        formatter.config.whitespace.max_width = 220;
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_struct_comments() {
        let sway_code_to_format = r#"contract;
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
// This is a comment
"#;
        let correct_sway_code = r#"contract;
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
// This is a comment
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_enum_comments() {
        let sway_code_to_format = r#"contract;
pub enum Bazz { // Here is a comment
    // Trying some ASCII art
    baz: (),





    bazzz: (),//-----
              //--D--
              //-----
}
"#;
        let correct_sway_code = r#"contract;
pub enum Bazz { // Here is a comment
    // Trying some ASCII art
    baz: (),
    bazzz: (),//-----
              //--D--
              //-----
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_fn_comments() {
        let sway_code_to_format = r#"contract;
// This is a comment before a fn
// This is another comment before a fn
fn hello_world( baz: /* this is a comment */ u64) { let x = 5; // This is a comment inside the block
}
"#;
        let correct_sway_code = r#"contract;
// This is a comment before a fn
// This is another comment before a fn
fn hello_world(baz: /* this is a comment */ u64) {
    let x = 5; // This is a comment inside the block
}
"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_abi_comments() {
        let sway_code_to_format = r#"contract;
// This is an abi
abi StorageMapExample {
    // insert_into_map is blah blah
    #[storage(write)] // this is some other comment
    fn insert_into_map(key: u64, value: u64);
    // this is the last comment inside the StorageMapExample
}"#;
        let correct_sway_code = r#"contract;
// This is an abi
abi StorageMapExample {
    // insert_into_map is blah blah
    #[storage(write)] // this is some other comment
    fn insert_into_map(key: u64, value: u64);
    // this is the last comment inside the StorageMapExample
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_const_comments() {
        let sway_code_to_format = r#"contract;
pub const /* TEST: blah blah tests */ TEST: u16 = 10; // This is a comment next to a const"#;
        let correct_sway_code = r#"contract;
pub const /* TEST: blah blah tests */ TEST: u16 = 10; // This is a comment next to a const
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_storage_comments() {
        let sway_code_to_format = r#"contract;

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
}"#;
        let correct_sway_code = r#"contract;

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
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_trait_comments() {
        let sway_code_to_format = r#"contract;
// This is the programmer trait
trait Programmer {
    // Returns fav languages of this Programmer.
    fn fav_language(self) -> String;
}"#;
        let correct_sway_code = r#"contract;
// This is the programmer trait
trait Programmer {
    // Returns fav languages of this Programmer.
    fn fav_language(self) -> String;
}
"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_where_comment() {
        let sway_code_to_format = r#"contract;

pub fn hello( person: String ) -> String where /* This is next to where */ T: Eq, /*Here is a comment*/{let greeting = 42;greeting.to_string()}"#;
        let correct_sway_code = r#"contract;

pub fn hello(person: String) -> String
where /* This is next to where */
    T: Eq, /*Here is a comment*/
{
    let greeting = 42;
    greeting.to_string()
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_impl() {
        let sway_code_to_format = r#"script;

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
    }}"#;
        let correct_sway_code = r#"script;

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
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_impl_without_generics() {
        let sway_code_to_format = r#"script;

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
    }}"#;
        let correct_sway_code = r#"script;

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
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_newline_sequence_formatting() {
        let sway_code_to_format = r#"script;

fn main() {
    let number: u64 = 10;

    let number2: u64 = 20;



    let number3: u64 = 30;



}"#;

        let correct_sway_code = r#"script;

fn main() {
    let number: u64 = 10;

    let number2: u64 = 20;

    let number3: u64 = 30;
}
"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_doc_comments() {
        let sway_code_to_format = r#"script;

enum Color {
    /// Blue color
    blue: (),
    /// Red color
    red: ()
}
/// This is the main function
fn main() {
}"#;

        let correct_sway_code = r#"script;

enum Color {
    /// Blue color
    blue: (),
    /// Red color
    red: (),
}
/// This is the main function
fn main() {}
"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }

    #[test]
    fn test_newline_comment_handler_interaction() {
        let sway_code_to_format = r#"script;

// use statements
use std::*;

fn main() {
    // Array of integers with type ascription
let array_of_integers: [u8;
5] = [1, 2, 3, 4, 5];


    // Array of strings
    let array_of_strings = [ "Bob", "Jan", "Ron"];
}
"#;

        let correct_sway_code = r#"script;

// use statements
use std::*;

fn main() {
    // Array of integers with type ascription
    let array_of_integers: [u8; 5] = [1, 2, 3, 4, 5];

    // Array of strings
    let array_of_strings = ["Bob", "Jan", "Ron"];
}
"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn comments_before_module_kind() {
        let sway_code_to_format = r#"// something about module kind
// something else about module kind
library test_module_kind_with_comments;"#;
        let correct_sway_code = r#"// something about module kind
// something else about module kind
library test_module_kind_with_comments;
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn newline_before_comments() {
        let sway_code_to_format = r#"


// something about module kind
// something else about module kind
library test_module_kind_with_comments;"#;
        let correct_sway_code = r#"// something about module kind
// something else about module kind
library test_module_kind_with_comments;
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_destructure_structs() {
        let sway_code_to_format = r#"library test_destructure_structs;

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
"#;
        let correct_sway_code = r#"library test_destructure_structs;

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
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_multiline_collections() {
        let sway_code_to_format = r#"library test_multiline_collections;
fn func_with_multiline_collections() {
    let x = (
        "hello",
        "world",
    );
}
"#;
        let correct_sway_code = r#"library test_multiline_collections;
fn func_with_multiline_collections() {
    let x = ("hello", "world");
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_parameterless_attributes() {
        let sway_code_to_format = r#"library my_lib;

abi MyContract {
    #[test]
    fn foo();
}
"#;
        let correct_sway_code = r#"library my_lib;

abi MyContract {
    #[test]
    fn foo();
}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
    #[test]
    fn test_multiple_comma_separated_attributes() {
        let sway_code_to_format = r#"library my_lib;

#[test, inline(always)]
fn foo() {}
"#;
        let correct_sway_code = r#"library my_lib;

#[test, inline(always)]
fn foo() {}
"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
        assert!(test_stability(formatted_sway_code, formatter));
    }
}
