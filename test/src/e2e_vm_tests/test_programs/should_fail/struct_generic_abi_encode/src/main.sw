library;

#[allow(dead_code)]
struct SomeStruct<T> {
    ptr: raw_ptr,
    cap: u64,
}

struct Buffer { }

trait AbiEncode2 {
    fn abi_encode2(self, ref mut buffer: Buffer);
}


impl AbiEncode2 for u64
{
    fn abi_encode2(self, ref mut buffer: Buffer) { }
}

impl<T> AbiEncode2 for SomeStruct<T> where T: AbiEncode2
{
    #[allow(dead_code)]
    fn abi_encode2(self, ref mut buffer: Buffer) {
        self.ptr.abi_encode2(buffer);
        self.cap.abi_encode2(buffer);
    }
}