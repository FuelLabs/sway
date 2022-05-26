library mapping;

use ::hash::*;
use core::storable::*;

pub struct Mapping<K> {
    seed_key: b256
}

impl core::storable::Storable for Mapping {
    fn write(self, key: b256) {
        self.seed_key.write(key);
    }
    fn read(key: b256) -> Self {
        Mapping {
            seed_key: ~b256::read(key),
        }
    }
}

// Will be generic
impl<K> Mapping<K> {
    fn new() -> Mapping<K> {
        Mapping { seed_key: __generate_b256_seed() }
    }

    fn insert(self, key: K, value: u64) {
        value.write(sha256((key, self.seed_key)));
    }

    fn get(self, key: K) -> u64 {
        ~u64::read(sha256((key, self.seed_key)))
    }
}
