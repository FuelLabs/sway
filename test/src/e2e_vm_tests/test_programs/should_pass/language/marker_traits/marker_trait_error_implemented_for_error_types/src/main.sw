library;

mod use_error_explicitly;
mod use_error_via_glob;

#[error_type]
enum Enum {
    #[error(m = "error message")]
    A: (),
}

// TODO: Enable these tests once trait coherence is implemented.
//       Currently, trait impls relay on order of implementation.
//       E.g., this will not compile, although trait `A` is implemented for `S`:
//          trait A {}
//          trait B: A {}
//          struct S {}
//          impl B for S {} <<-- Trait "A" is not implemented for type "S".
//          impl A for S {}
//       Since auto-implemented `Error` trait gets inserted before
//       any manually implemented `AbiEncode` traits, it will never see that
//       a manual `AbiEncode` impl exists for the enum.
// #[error_type]
// enum EnumWithNoAutoImplementedAbiEncode {
//     #[error(m = "error message")]
//     A: raw_ptr,
// }

// impl AbiEncode for EnumWithNoAutoImplementedAbiEncode {
//     fn abi_encode(self, buffer: Buffer) -> Buffer {
//         buffer
//     }
// }

// #[error_type]
// enum EnumWithCustomAbiEncode {
//     #[error(m = "error message")]
//     A: (),
// }

// impl AbiEncode for EnumWithCustomAbiEncode {
//     fn abi_encode(self, buffer: Buffer) -> Buffer {
//         buffer
//     }
// }

// Using `Error` from the standard library prelude.
fn implements_error<T>(_t: T) where T: Error { }
fn implements_error_no_args<T>() where T: Error { }

pub fn main() {
    implements_error("str");
    implements_error_no_args::<str>();
    implements_error(());
    implements_error_no_args::<()>();
    implements_error(Enum::A);
    implements_error_no_args::<Enum>();
    // TODO: Enable these tests once trait coherence is implemented.
    // implements_error(EnumWithNoAutoImplementedAbiEncode::A(__addr_of(Enum::A)));
    // implements_error_no_args::<EnumWithNoAutoImplementedAbiEncode>();
    // implements_error(EnumWithCustomAbiEncode::A);
    // implements_error_no_args::<EnumWithCustomAbiEncode>();
    use_error_explicitly::test();
    use_error_via_glob::test();
}
