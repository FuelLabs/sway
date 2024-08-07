library;

pub struct Buffer {
    buffer: u64
}

pub trait T {
    fn ar(buffer: Buffer) -> Buffer;
}

impl T for str[10] {
    fn ar(buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer)
        }
    }
}