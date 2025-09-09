library;

pub struct StorageFoobar {}

impl StorageKey<StorageFoobar> {
    pub fn foobar(self) {
        log("foobar");
    }
}