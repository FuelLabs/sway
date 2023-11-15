library; 

pub struct Buffer {
    buffer: raw_ptr,
    cap: u64,
    size: u64
}

impl Buffer {
    pub fn new() -> Self {
        let cap = 1024;
        Buffer {
            buffer: asm(size: cap) {
                aloc size;
                hp: raw_ptr
            },
            cap,
            size: 0
        }
    }

    pub fn push<T>(ref mut self, val: T) {
        let count = __size_of::<T>();

        if self.cap >= self.size + count {
            if __is_reference_type::<T>() {
                asm(dst: self.buffer, val: val, count: count) {
                    mcp dst val count;
                };
            } else {
                asm(ptr: self, val: val) {
                    sw ptr val i0;
                };
            };

            self.size += count;
        } else {

        }
        
    }
}

pub trait AbiEncoder {
    fn abi_encode(self, ref mut buffer: Buffer);
}

impl AbiEncoder for u64 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        buffer.push(self);
    }
}

pub fn encode<T>(item: T) where T: AbiEncoder {
    let mut buffer = Buffer::new();
    item.abi_encode(buffer);
}
