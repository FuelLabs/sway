contract;

mod foobar;

use foobar::StorageFoobar;

storage {
    foo: StorageFoobar = StorageFoobar {},
}

impl Contract {
    fn foobar() {
        storage.foo.foobar();
    }
}