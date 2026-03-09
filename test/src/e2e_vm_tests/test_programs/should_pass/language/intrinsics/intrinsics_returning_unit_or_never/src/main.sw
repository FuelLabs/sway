// This test proves that https://github.com/FuelLabs/sway/issues/7572 is fixed.

contract;

impl Contract {
    #[storage(write)]
    fn check_proper_returns() {
        poke(__assert_is_str_array::<str[1]>());
        poke(__assert_is_str_array::<str[1]>() == ());
        poke(__smo(b256::zero(), 0u64, 0));
        poke(__smo(b256::zero(), 0u64, 0) == ());
        poke(__log(0u64));
        poke(__log(0u64) == ());
        poke(__state_clear_slots(b256::zero(), 1));
        poke(__state_clear_slots(b256::zero(), 1) == ());
        poke(__state_clear(b256::zero(), 1) == true);
        poke(__state_store_slot(b256::zero(), __addr_of(0), 0));
        poke(__state_store_slot(b256::zero(), __addr_of(0), 0) == ());
        poke(__revert(0u64));
        poke(__revert(0u64) == { return; });
    }
}

#[inline(never)]
fn poke<T>(_x: T) { }
