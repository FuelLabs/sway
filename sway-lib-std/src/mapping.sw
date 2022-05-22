library mapping;

use ::hash::*;
use core::storable::*;

pub struct Mapping {
    seed_key: b256
}

impl core::storable::Storable for Mapping {
    fn write(self, key: b256) {
        self.seed_key.write(key);
    }
    fn read(key: b256) -> Mapping {
        Mapping {
            seed_key: ~b256::read(key),
        }
    }
}

// Will be generic
impl Mapping {
    fn new() -> Mapping {
        Mapping { seed_key: __generate_b256_seed() }
    }

    fn insert(self, key: u64, value: u64) {
        value.write(sha256((key, self.seed_key)));
    }

    fn get(self, key: u64) -> u64 {
        ~u64::read(sha256((key, self.seed_key)))
    }
}
