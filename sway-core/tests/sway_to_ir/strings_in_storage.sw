contract;

abi StorageAccess {
    // Setters
    fn set_s(s: str[40]);
    fn get_s() -> str[40];
}

storage {
    s: str[40],
}

impl StorageAccess for Contract {
    fn set_s(s: str[40]) {
        storage.s = s;
    }

    fn get_s() -> str[40] {
        storage.s
    }
}
