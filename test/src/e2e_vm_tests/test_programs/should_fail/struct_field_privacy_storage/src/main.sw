contract;

mod lib;

use lib::*;

struct MainStruct {
    pub x: u64,
    y: u64,
    other: MainOtherStruct,
}

impl MainStruct {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
        poke(self.other);
    }
}

struct MainOtherStruct {
    pub x: u64,
    y: u64,
}

impl MainOtherStruct {
    fn use_me(self) {
        poke(self.x);
        poke(self.y);
    }
}

storage {
    ms: MainStruct = MainStruct { x: 0, y: 0, other: MainOtherStruct { x: 0, y: 0 } },
    ls: LibStruct = LibStruct { },
}

abi AccessStorage {
    #[storage(read)]
    fn access_storage();
}

impl AccessStorage for Contract {
    #[storage(read)]
    fn access_storage() {
        let _ = storage.ls.x.read();
        let _ = storage.ls.y.read();
        let _ = storage.ls.other.read();
        let _ = storage.ls.other.x.read();
        let _ = storage.ls.other.y.read();
        
        let _ = storage.ms.x.read();
        let _ = storage.ms.y.read();
        let _ = storage.ms.other.read();
        let _ = storage.ms.other.x.read();
        let _ = storage.ms.other.y.read();

        let ms = MainStruct { x: 0, y: 0, other: MainOtherStruct { x: 0, y: 0 } };
        ms.use_me();
        ms.other.use_me();
    }
}

fn poke<T>(_x: T) { }