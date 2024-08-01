contract;

abi A {
    const A_WITH_DEFAULT: u32 = 3;
    // TODO: Uncomment this case when https://github.com/FuelLabs/sway/issues/6348 is fixed.
    // const A_NO_DEFAULT: u32;
    const COMMON_1: u32 = 5;
    // TODO: Uncomment this version (without default) when https://github.com/FuelLabs/sway/issues/6348 is fixed.
    // const COMMON_2: u32;
    const COMMON_2: u32 = 111; // TODO: Remove this one in favor of the one without default.

    fn a_with_default() -> u32;
    // fn a_no_default() -> u32;

    fn common_1() -> u32;
    fn common_2() -> u32;
} {
    // TODO: Uncomment this case once `expression_variant::find_const_decl_from_impl` is implemented.
    // fn a_implemented_with_default() -> u32 {
    //     Self::A_WITH_DEFAULT
    // }

    // fn a_implemented_no_default() -> u32 {
    //     Self::A_NO_DEFAULT
    // }

    // fn a_implemented_common_1() -> u32 {
    //     Self::COMMON_1
    // }

    // fn a_implemented_common_2() -> u32 {
    //     Self::COMMON_2
    // }
}
    
abi B {
    // TODO: Uncomment this version (without default) when https://github.com/FuelLabs/sway/issues/6348 is fixed.
    // const COMMON_1: u32;
    const COMMON_1: u32 = 111; // TODO: Remove this one in favor of the one without default.
    const COMMON_2: u32 = 7;

    fn common_1() -> u32;
    fn common_2() -> u32;
} {
    // TODO: Uncomment this case once `expression_variant::find_const_decl_from_impl` is implemented.
    // fn b_implemented_common_1() -> u32 {
    //     Self::COMMON_1
    // }

    // fn b_implemented_common_2() -> u32 {
    //     Self::COMMON_2
    // }
}

impl A for Contract {
    const A_WITH_DEFAULT: u32 = 13;
    // const A_NO_DEFAULT: u32 = 133;
    const COMMON_1: u32 = 15;
    const COMMON_2: u32 = 155;

    fn a_with_default() -> u32 {
        Self::A_WITH_DEFAULT
    }

    // fn a_no_default() -> u32 {
    //     Self::A_NO_DEFAULT
    // }

    fn common_1() -> u32 {
        Self::COMMON_1
    }

    fn common_2() -> u32 {
        Self::COMMON_2
    }
}

impl B for Contract {
    const COMMON_1: u32 = 177;
    const COMMON_2: u32 = 17;

    fn common_1() -> u32 {
        Self::COMMON_1
    }

    fn common_2() -> u32 {
        Self::COMMON_2
    }
}

#[test]
fn test() {
    let a = abi(A, CONTRACT_ID);
    // TODO: Enable these asserts once these bugs are fixed:
    //       https://github.com/FuelLabs/sway/issues/6310
    //       https://github.com/FuelLabs/sway/issues/6306
    //       https://github.com/FuelLabs/sway/issues/6348

    // assert_eq(13, a.a_with_default());
    // assert_eq(13, a.a_implemented_with_default());

    // assert_eq(133, a.a_no_default());
    // assert_eq(133, a.a_implemented_no_default());

    // assert_eq(15, a.common_1());
    // assert_eq(15, a.a_implemented_common_1());

    // assert_eq(155, a.common_2());
    // assert_eq(155, a.a_implemented_common_2());

    let b = abi(B, CONTRACT_ID);

    // assert_eq(177, b.common_1());
    // assert_eq(177, b.b_implemented_common_1());

    // assert_eq(17, b.common_2());
    // assert_eq(17, b.b_implemented_common_2());
}
