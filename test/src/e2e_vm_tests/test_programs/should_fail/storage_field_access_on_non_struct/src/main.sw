contract;

abi ReadStorage {
    #[storage(read)]
    fn read_storage();
}

struct Struct01 {
    x: u8,
    second: Struct02,
}

impl Struct01 {
    fn use_me(self) {
        poke(self.x);
        poke(self.second);
    }
}

struct Struct02 {
    x: u32,
    third: Struct03,
}

impl Struct02 {
    fn use_me(self) {
        poke(self.x);
        poke(self.third);
    }
}

struct Struct03 {
    x: u64,
}

impl Struct03 {
    fn use_me(self) {
        poke(self.x);
    }
}

storage {
    b: bool = true,
    s_01: Struct01 = Struct01 { x: 0, second: Struct02 { x: 0, third: Struct03 { x: 0 } } },
}

impl ReadStorage for Contract {
    #[storage(read)]
    fn read_storage() {
        let _ = storage.not_in_storage.read();

        let _ = storage.b.read();

        let _ = storage.b.prev_not_a_struct.read();
        
        let s_01 = storage.s_01.read();
        let _ = storage.s_01.x.read();

        let _ = storage.s_01.x.prev_not_a_struct.read();
        let _ = storage.s_01.non_existing_field.read();

        let s_02 = storage.s_01.second.read();
        let _ = storage.s_01.second.x.read();
        
        let _ = storage.s_01.second.x.prev_not_a_struct.read();
        let _ = storage.s_01.second.non_existing_field.read();

        let s_03 = storage.s_01.second.third.read();
        let _ = storage.s_01.second.third.x.read();
        
        let _ = storage.s_01.second.third.x.prev_not_a_struct.read();
        let _ = storage.s_01.second.third.non_existing_field.read();

        s_01.use_me();
        s_02.use_me();
        s_03.use_me();
    }
}

fn poke<T>(_x: T) { }
