contract;

abi CalledContract {
    #[storage(write)]
    fn set_value(new_value: u8);
    #[storage(read)]
    fn get_value() -> u8;

}

 storage {
    value: u8 = 0,
    }


impl CalledContract for Contract {
    #[storage(write)]
    fn set_value(new_value: u8) {
        storage.value = new_value;
    }

    #[storage(read)]
    fn get_value() -> u8{
        storage.value
    }
}
