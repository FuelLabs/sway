library mapping;

use ::hash::sha256;
use ::storage::{get, store};

pub struct Mapping<K, V> {
    seed: b256,
}

impl<K, V> Mapping<K, V> {
    fn new() -> Mapping<K, V> {
        Mapping { seed: __generate_uid() }
    }

    fn insert(self, key: K, value: V) {
        let key = sha256((key, self.seed));
        store::<V>(key, value);
    }

    fn get(self, key: K) -> V {
        let key = sha256((key, self.seed));
        get::<V>(key)
    }
}
