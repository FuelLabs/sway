// TODO: Extend the test with other elements once https://github.com/FuelLabs/sway/issues/6942 is implemented.
// TODO: Extend the test with usages in all expressions.
contract;

#[deprecated(note = "Use \"NonDeprecatedStruct\" instead.")]
struct DeprecatedStruct {
    a: u64,
}

struct NonDeprecatedStruct {
    #[deprecated]
    deprecated_field: u64,
}

impl NonDeprecatedStruct {
    #[deprecated]
    fn deprecated_method(self) {}
    #[deprecated]
    fn deprecated_assoc_fun(self) {}
    #[deprecated]
    const DEPRECATED_ASSOC_CONST: u64 = 0;
}

#[deprecated(note = "Use \"NonDeprecatedEnum\" instead.")]
enum DeprecatedEnum {
    A: u64,
}

enum NonDeprecatedEnum {
    #[deprecated]
    DeprecatedVariant: u64,
    A: u64,
}

impl NonDeprecatedEnum {
    #[deprecated]
    fn deprecated_method(self) {}
    #[deprecated]
    fn deprecated_assoc_fun(self) {}
    // TODO: Enable once https://github.com/FuelLabs/sway/issues/6344 is implemented.
    // #[deprecated]
    // const DEPRECATED_ASSOC_ENUM_CONST: u64 = 0;
}

configurable {
    #[deprecated]
    DEPRECATED: u64 = 0,
}

#[deprecated(note = "Use \"non_deprecated\" instead.")]
fn deprecated(_d: DeprecatedStruct) {}

abi NonDeprecatedAbi {
    fn deprecated_to_be_abi_method();
} {
    #[deprecated]
    fn deprecated_abi_provided_method() {}
}

impl NonDeprecatedAbi for Contract {
    #[deprecated]
    fn deprecated_to_be_abi_method() {}
}

trait NonDeprecatedTrait {
    fn deprecated_to_be_trait_method(self);
    fn deprecated_to_be_trait_assoc_fun();
} {
    #[deprecated]
    fn deprecated_trait_provided_method(self) {}
    #[deprecated]
    fn deprecated_trait_provided_assoc_fun() {}
}

impl NonDeprecatedTrait for NonDeprecatedStruct {
    #[deprecated]
    fn deprecated_to_be_trait_method(self) {}
    #[deprecated]
    fn deprecated_to_be_trait_assoc_fun() {}
}

pub fn call_deprecated() {
    let ds = DeprecatedStruct { a: 0 };
    let _ = ds.a;

    let mut nds = NonDeprecatedStruct { deprecated_field: 0 };
    let _ = nds.deprecated_field;
    let r_nds = &mut nds;
    (*r_nds).deprecated_field = 1;
    nds.deprecated_method();
    nds.deprecated_assoc_fun();
    let _ = NonDeprecatedStruct::DEPRECATED_ASSOC_CONST;

    let _ = DeprecatedEnum::A(0);
    let nde = NonDeprecatedEnum::DeprecatedVariant(0);
    nde.deprecated_method();
    nde.deprecated_assoc_fun();
    // TODO: Enable once https://github.com/FuelLabs/sway/issues/6344 is implemented.
    // let _ = NonDeprecatedEnum::DEPRECATED_ASSOC_CONST;

    deprecated(ds);

    let _ = DEPRECATED;

    let caller = abi(NonDeprecatedAbi, b256::zero());
    caller.deprecated_abi_provided_method();
    caller.deprecated_to_be_abi_method();

    nds.deprecated_to_be_trait_method();
    nds.deprecated_trait_provided_method();
    NonDeprecatedStruct::deprecated_to_be_trait_assoc_fun();
    NonDeprecatedStruct::deprecated_trait_provided_assoc_fun();

    match ds {
        DeprecatedStruct { .. } => { }
    }

    match nds {
        NonDeprecatedStruct { deprecated_field, .. } => {
            let _ = deprecated_field;
        }
    }
}
