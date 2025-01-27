library;

pub mod enums_01;
pub mod enums_02;
mod use_error_explicitly;
mod use_error_via_glob;

use enums_01::{LibEnum01, EmptyLibEnum01};

// TODO: Currently, trait impls will be imported into namespace
//       only if the implemented for type is imported and they
//       reside in the same module, or if glob imported.
//
//       In the test, all `enums_02` enums are accessed using the
//       full path. Remove this `use` once trait coherence and
//       collecting of trait impls is implemented:
//       https://github.com/FuelLabs/sway/issues/5892
use enums_02::*;

enum MainEnum {
    A: (),
}

enum MainEmptyEnum { }

// Using `Enum` from core library prelude.
fn implements_enum<T>(_t: T) where T: Enum { }
fn implements_enum_no_args<T>() where T: Enum { }

pub fn main() {
    implements_enum(MainEnum::A);
    implements_enum_no_args::<MainEnum>();
    implements_enum_no_args::<MainEmptyEnum>();

    implements_enum(LibEnum01::A);
    implements_enum_no_args::<LibEnum01>();
    implements_enum_no_args::<EmptyLibEnum01>();

    implements_enum(enums_02::LibEnum02::A);
    implements_enum_no_args::<enums_02::LibEnum02>();
    implements_enum_no_args::<enums_02::EmptyLibEnum02>();

    use_error_explicitly::test();
    use_error_via_glob::test();
}
