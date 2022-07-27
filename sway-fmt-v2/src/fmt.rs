use crate::utils::{
    comments::handle_comments, indent_style::Shape, newline_style::apply_newline_style,
    program_type::insert_program_type,
};
use std::{path::Path, sync::Arc};
use sway_core::BuildConfig;

pub use crate::{
    config::manifest::Config,
    error::{ConfigError, FormatterError},
};

#[derive(Debug, Default)]
pub struct Formatter {
    pub(crate) shape: Shape,
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
        let shape = Shape::default();
        Ok(Self { config, shape })
    }
    pub fn format(
        &mut self,
        src: Arc<str>,
        build_config: Option<&BuildConfig>,
    ) -> Result<FormattedCode, FormatterError> {
        let path = build_config.map(|build_config| build_config.canonical_root_module());
        let src_len = src.len();
        let module = sway_parse::parse_file(src.clone(), path.clone())?;
        // Get parsed items
        let items = &module.items;
        // Get the program type (script, predicate, contract or library)
        let program_type = &module.kind;

        // Formatted code will be pushed here with raw newline stlye.
        // Which means newlines are not converted into system-specific versions until `apply_newline_style()`.
        // Use the length of src as a hint of the memory size needed for `raw_formatted_code`,
        // which will reduce the number of reallocations
        let mut raw_formatted_code = String::with_capacity(src_len);

        // Insert program type to the formatted code.
        insert_program_type(&mut raw_formatted_code, program_type)?;

        // Insert parsed & formatted items into the formatted code.
        let mut iter = items.iter().peekable();
        while let Some(item) = iter.next() {
            // format Annotated<ItemKind>
            item.format(&mut raw_formatted_code, self)?;
            if iter.peek().is_some() {
                raw_formatted_code.push('\n');
            }
        }

        let mut formatted_code = String::from(&raw_formatted_code);
        // Add comments
        handle_comments(
            src,
            &module,
            Arc::from(formatted_code.clone()),
            path,
            &mut formatted_code,
        )?;
        // Replace newlines with specified `NewlineStyle`
        apply_newline_style(
            self.config.whitespace.newline_style,
            &mut formatted_code,
            &raw_formatted_code,
        )?;
        if formatted_code.ends_with('\n') {
            formatted_code.pop();
        }

        Ok(formatted_code)
    }
}

#[cfg(test)]
mod tests {
    use super::Formatter;
    use crate::config::user_def::FieldAlignment;
    use std::sync::Arc;

    #[test]
    fn test_const() {
        let sway_code_to_format = r#"contract;
pub const TEST:u16=10;"#;
        let correct_sway_code = r#"contract;

pub const TEST: u16 = 10;"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
    }

    #[test]
    fn test_struct_multiline_line_alignment() {
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
}"#;

        let mut formatter = Formatter::default();
        formatter.config.structures.field_alignment = FieldAlignment::AlignFields(40);
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
    }
    #[test]
    fn test_struct_single_line() {
        let sway_code_to_format = r#"contract;
pub struct Foo {
    bar: u64,
    baz: bool,
}
"#;
        let correct_sway_code = r#"contract;

pub struct Foo { bar: u64, baz: bool }"#;
        let mut formatter = Formatter::default();
        formatter.config.structures.small_structures_single_line = true;
        formatter.config.whitespace.max_width = 300;
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
    }
    #[test]
    fn test_enum_single_line() {
        let sway_code_to_format = r#"contract;
pub enum Foo {
    bar: u64,
    baz: bool,
}
"#;
        let correct_sway_code = r#"contract;

pub enum Foo { bar: u64, baz: bool }"#;
        let mut formatter = Formatter::default();
        formatter.config.structures.small_structures_single_line = true;
        formatter.config.whitespace.max_width = 300;
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
    }
    #[test]
    fn test_struct_multi_line() {
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
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
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
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
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
}"#;

        // Creating a config with enum_variant_align_threshold that exceeds longest variant length
        let mut formatter = Formatter::default();
        formatter.config.structures.field_alignment = FieldAlignment::AlignFields(20);

        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
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
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
    }

    #[test]
    fn test_multi_items() {
        let sway_code_to_format = r#"contract;

pub const TEST: u16 = 10;
pub const TEST1: u16 = 10;"#;
        let correct_sway_code = r#"contract;

pub const TEST: u16 = 10;
pub const TEST1: u16 = 10;"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
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
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
    }
    fn test_storage_without_alignment() {
        let sway_code_to_format = r#"contract;

storage{foo:Test=Test{},bar
: 
    Test=Test{}
, baz: u64 } 
"#;
        let correct_sway_code = r#"contract;

storage {
    foo: Test,
    bar: Test,
    baz: u64,
}"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
    }
    #[test]
    fn test_storage_with_alignment() {
        let sway_code_to_format = r#"contract;

storage {
 long_var_name: Type1=Type1{},
      var2: Type2=Type2{},
}
"#;
        let correct_sway_code = r#"contract;

storage {
    long_var_name : Type1 = Type1 {
    },
    var2          : Type2 = Type2 {
    },
}"#;

        let mut formatter = Formatter::default();
        formatter.config.structures.field_alignment = FieldAlignment::AlignFields(50);
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
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
    var1: Type1 = Type1 {x: 0,y: 0, },
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
    var1: Type1 = Type1 {
        x: 0,
        y: 0,
    },
    var2: Type2 = Type2 {
        w: 0x0000000000000000000000000000000000000000000000000000000000000000,
        z: false,
    },
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
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
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
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
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
    }
    #[test]
    fn test_trait_and_super_trait() {
        let sway_code_to_format = r#"library traits;

trait Person{ fn name( self )->String;fn age( self )->usize; }
trait Student:Person {fn university(self) -> String;}
trait Programmer {fn fav_language(self) -> String;}
trait CompSciStudent: Programmer+Student {fn git_username(self) -> String;}"#;
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
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
    }

    #[test]
    fn test_struct_comments() {
        let sway_code_to_format = r#"contract;
// This is a comment, for this one to be placed correctly we need to have Module visitor implemented
pub struct Foo { // Here is a comment



    // Trying some ASCII art
    baz:u64,




    bazzz:u64//  ________ ___  ___  _______   ___               ___       ________  ________  ________
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
    bazzz: u64,//  ________ ___  ___  _______   ___               ___       ________  ________  ________
             // |\  _____\\  \|\  \|\  ___ \ |\  \             |\  \     |\   __  \|\   __  \|\   ____\
             // \ \  \__/\ \  \\\  \ \   __/|\ \  \            \ \  \    \ \  \|\  \ \  \|\ /\ \  \___|_
             //  \ \   __\\ \  \\\  \ \  \_|/_\ \  \            \ \  \    \ \   __  \ \   __  \ \_____  \
             //   \ \  \_| \ \  \\\  \ \  \_|\ \ \  \____        \ \  \____\ \  \ \  \ \  \|\  \|____|\  \
             //    \ \__\   \ \_______\ \_______\ \_______\       \ \_______\ \__\ \__\ \_______\____\_\  \
             //     \|__|    \|_______|\|_______|\|_______|        \|_______|\|__|\|__|\|_______|\_________\
             //                                                                                  \|_________|
}
// This is a comment"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
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
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
    }

    #[test]
    fn test_fn_comments() {
        let sway_code_to_format = r#"contract;
// This is a comment before a fn
// This is another comment before a fn
fn hello_world( baz: /* this is a comment */ u64) { // This is a comment inside the block
}
"#;
        let correct_sway_code = r#"contract;

// This is a comment before a fn
// This is another comment before a fn
fn hello_world(baz: /* this is a comment */ u64) { // This is a comment inside the block
}"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
    }

    #[test]
    fn test_abi_comments() {
        let sway_code_to_format = r#"contract;
// This is an abi
abi StorageMapExample {
    // insert_into_map is blah blah
    #[storage(write)] // this is some other comment
    fn insert_into_map(key: u64, value: u64); // this is the last comment inside the StorageMapExample
}"#;
        let correct_sway_code = r#"contract;

// This is an abi
abi StorageMapExample {
    // insert_into_map is blah blah
    #[storage(write)] // this is some other comment
    fn insert_into_map(key: u64, value: u64); // this is the last comment inside the StorageMapExample
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
    }

    #[test]
    fn test_const_comments() {
        let sway_code_to_format = r#"contract;
pub const /* TEST: blah blah tests */ TEST: u16 = 10; // This is a comment next to a const"#;
        let correct_sway_code = r#"contract;

pub const /* TEST: blah blah tests */ TEST: u16 = 10;"#; // Comment next to const is not picked up by the lexer see: #2356
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
    }
    #[test]
    fn test_storage_comments() {
        let sway_code_to_format = r#"contract;
storage {
    // Testing a comment inside storage
    long_var_name: Type1=Type1{},
    // Testing another comment
    var2: Type2 = Type2{} // This is the last comment
}"#;
        let correct_sway_code = r#"contract;

storage {
    // Testing a comment inside storage
    long_var_name: Type1 = Type1 {
    },
    // Testing another comment
    var2: Type2 = Type2 {
    }, // This is the last comment
}"#;
        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code);
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
}"#;

        let mut formatter = Formatter::default();
        let formatted_sway_code =
            Formatter::format(&mut formatter, Arc::from(sway_code_to_format), None).unwrap();
        assert_eq!(correct_sway_code, formatted_sway_code)
    }
}
