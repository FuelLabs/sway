script;

trait AbiEncode2 {
    fn abi_encode2(self, ref mut buffer: Buffer);
}

impl AbiEncode2 for u64 { fn abi_encode2(self, ref mut buffer: Buffer) { } }
impl AbiEncode2 for u32 { fn abi_encode2(self, ref mut buffer: Buffer) { } }

struct GenericBimbam<U> {
    val: U,
}

impl<U> AbiEncode2 for GenericBimbam<U> where U: AbiEncode2
 {
    fn abi_encode2(self, ref mut buffer: Buffer) {
        self.val.abi_encode2(buffer);
    }
}

struct GenericSnack<T, V> {
    twix: GenericBimbam<T>,
    mars: V,
}

impl<T, V> AbiEncode2 for GenericSnack<T, V> where T: AbiEncode2, V: AbiEncode2
 {
    fn abi_encode2(self, ref mut buffer: Buffer) {
        self.twix.abi_encode2(buffer);
        self.mars.abi_encode2(buffer);
    }
}

fn encode2<T>(item: T) -> raw_slice where T: AbiEncode2  {
    let mut buffer = Buffer::new();
    item.abi_encode2(buffer);
    buffer.as_raw_slice()
}

fn main() {
    encode2(GenericSnack { twix: GenericBimbam { val: 2u64 }, mars: 2u32 });
}