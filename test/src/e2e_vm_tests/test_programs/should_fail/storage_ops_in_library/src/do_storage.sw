library do_storage;

const KEY: b256 = 0xfafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafa;

#[storage(read, write)]
pub fn side_effects() {
    asm(key: KEY, v) {
        srw v key;
        sww key v;
    }
}
