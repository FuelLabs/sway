library;

struct Buffer {
    buffer: (raw_ptr, u64, u64),
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buffer: __encode_buffer_empty(self),
        }
    }
}