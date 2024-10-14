contract;

struct S {}

impl S {
    fn bar() -> u8 {
        11
    }
}

impl S {
    // TODO: Uncomment this once https://github.com/FuelLabs/sway/issues/6538 is fixed.
    // const S_BAR: u8 = Self::bar();
    const S_A: u8 = 11;
    const S_B: u8 = Self::S_A;
    const S_C: u8 = 22;

    fn assoc_s_b() -> u8 {
        Self::S_B
    }

    fn assoc_s_c() -> u8 {
        Self::S_C
    }

    fn method_s_c(self) -> u8 {
        Self::S_C
    }
}

storage {
    // TODO: Uncomment this once https://github.com/FuelLabs/sway/issues/6543 is fixed.
    // s_a: u8 = S::S_A,
}

const MOD_S_A: u8 = S::S_A;

fn mod_fn_s_a() -> u8 {
    S::S_A
}

abi Abi {
    fn test_in_contract();
}

configurable {
    CONFIG_S_A: u8 = S::S_A,
}

impl Abi for Contract {
    fn test_in_contract() {
        assert_eq(11, S::S_A);
        assert_eq(22, S::S_C);

        // TODO: Uncomment this once https://github.com/FuelLabs/sway/issues/6544 is fixed.
        // assert_eq(S::S_B, S::S_A);
        // assert_eq(S::assoc_s_b(), S::S_B);
        assert_eq(S::assoc_s_c(), S::S_C);
        assert_eq(S {}.method_s_c(), S::S_C);

        // TODO: Uncomment this once https://github.com/FuelLabs/sway/issues/6538 is fixed.
        // assert_eq(S::bar(), S::S_BAR);

        assert_eq(S::S_A, MOD_S_A);

        assert_eq(S::S_A, mod_fn_s_a());

        assert_eq(S::S_A, CONFIG_S_A);

        // TODO: Uncomment this once https://github.com/FuelLabs/sway/issues/6543 is fixed.
        // assert_eq(S::S_A, storage.s_a.read());
    }
}

#[test]
fn test() {
    assert_eq(11, S::S_A);
    assert_eq(22, S::S_C);

    // TODO: Uncomment this once https://github.com/FuelLabs/sway/issues/6544 is fixed.
    // assert_eq(S::S_B, S::S_A);
    // assert_eq(S::assoc_s_b(), S::S_B);
    assert_eq(S::assoc_s_c(), S::S_C);
    assert_eq(S {}.method_s_c(), S::S_C);

    // TODO: Uncomment this once https://github.com/FuelLabs/sway/issues/6538 is fixed.
    // assert_eq(S::bar(), S::S_BAR);

    assert_eq(S::S_A, MOD_S_A);

    assert_eq(S::S_A, mod_fn_s_a());

    let caller = abi(Abi, CONTRACT_ID);
    caller.test_in_contract();
}
