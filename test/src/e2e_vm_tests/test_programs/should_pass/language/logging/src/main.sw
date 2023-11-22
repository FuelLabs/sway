script;

use core::codec::*;

struct ShouldBeAbiEncoder {
    a: u64
}

trait A {
    fn b();
}

impl A for ShouldBeAbiEncoder {
    fn b() {
        
    }
}

fn ff<T>(v: T) -> T where T: A {
    v
}

//impl AbiEncoder for ShouldBeAbiEncoder {
//    fn abi_encode(self, ref mut buffer: Buffer) {
//        buffer.push(self.a);
//    }
//}

fn main() {
    __log(1u64);
    __log(ff(ShouldBeAbiEncoder {
        a: 1
    }))
}
