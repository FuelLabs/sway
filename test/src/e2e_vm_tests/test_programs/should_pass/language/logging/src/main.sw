script;

use core::codec::*;

struct ShouldBeAbiEncoder {
    a: u64
}

//impl AbiEncoder for ShouldBeAbiEncoder {
//    fn abi_encode(self, ref mut buffer: Buffer) {
//        buffer.push(self.a);
//    }
//}

fn main() {
    __log(1u64);
    __log(ShouldBeAbiEncoder {
        a: 1
    })
}
