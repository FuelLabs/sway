library do_storage;

const KEY: b256 = 0xfafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafa;

#[storage(read, write)]
pub fn side_effects() {
    asm(key: KEY, is_set, v) {
        srw v is_set key;
        sww key is_set v;
    }
}
