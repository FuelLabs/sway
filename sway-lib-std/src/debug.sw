library;

use ::primitive_conversions::*;
use ::raw_ptr::*;
use ::codec::*;

// ssize_t write(int fd, const void buf[.count], size_t count);
fn syscall_write(fd: u64, buf: raw_ptr, count: u64) {
    asm(id: 1000, fd: fd, buf: buf, count: count) {
        ecal id fd buf count;
    }
}

pub struct DebugStruct {
    f: Formatter,
    has_fields: bool,
}

pub struct DebugList {
    f: Formatter,
    has_entries: bool,
}

pub struct DebugTuple {
    f: Formatter,
    has_fields: bool,
}

pub struct Formatter {
}

impl Formatter {
    pub fn print_newline(self) {
        let lf = [10u8];
        syscall_write(0, __addr_of(lf), 1);
    }

    pub fn print_str(self, s: str) {
        syscall_write(0, s.as_ptr(), s.len());
    }

    pub fn print_u8(self, value: u8) {
        let mut value = value;
        let mut digits = [0u8; 64];
        let mut i = 63; 
    
        while value > 0 { 
            let digit = value % 10; 
            digits[i] = digit + 48; // ascii zero = 48 
            i -= 1; 
            value = value / 10; 
        }

        syscall_write(0, __addr_of(digits).add::<u8>(i), 64 - i);
    }

    pub fn print_u16(self, value: u16) {
        let mut value = value;
        let mut digits = [0u8; 64];
        let mut i = 63; 
    
        while value > 0 { 
            let digit = asm(v: value % 10) {
                v: u8
            };
            digits[i] = digit + 48; // ascii zero = 48 
            i -= 1; 
            value = value / 10; 
        }

        syscall_write(0, __addr_of(digits).add::<u8>(i), 64 - i);
    }

    pub fn print_u32(self, value: u32) {
        let mut value = value;
        let mut digits = [0u8; 64];
        let mut i = 63; 
    
        while value > 0 { 
            let digit = asm(v: value % 10) {
                v: u8
            };
            digits[i] = digit + 48; // ascii zero = 48 
            i -= 1; 
            value = value / 10; 
        }

        syscall_write(0, __addr_of(digits).add::<u8>(i), 64 - i);
    }

    pub fn print_u64(self, value: u64) {
        let mut value = value;
        let mut digits = [0u8; 64];
        let mut i = 63; 
    
        while value > 0 { 
            let digit = asm(v: value % 10) {
                v: u8
            };
            digits[i] = digit + 48; // ascii zero = 48 
            i -= 1; 
            value = value / 10; 
        }

        syscall_write(0, __addr_of(digits).add::<u8>(i), 64 - i);
    }

    pub fn debug_struct(self, name: str) -> DebugStruct {
        self.print_str(name);
        self.print_str(" { ");
        DebugStruct { f: self, has_fields: false }
    }

    pub fn debug_list(self) -> DebugList {
        self.print_str("[");
        DebugList { f: self, has_entries: false }
    }

    pub fn debug_tuple(self) -> DebugTuple {
        self.print_str("(");
        DebugTuple { f: self, has_fields: false }
    }
}

impl DebugStruct {
    pub fn finish(ref mut self) {
        if self.has_fields {
            self.f.print_str(" ");    
        }
        self.f.print_str("}");
    }

    pub fn field<T>(ref mut self, name: str, value: T) -> Self
    where
        T: Debug 
    {
        if self.has_fields {
            self.f.print_str(", ");    
        }

        self.f.print_str(name);
        self.f.print_str(": ");
        value.fmt(self.f);

        self.has_fields = true;

        self
    }
}

impl DebugList {
    pub fn finish(ref mut self) {
        self.f.print_str("]");
    }

    pub fn entry<T>(ref mut self, value: T) -> Self
    where
        T: Debug 
    {
        if self.has_entries {
            self.f.print_str(", ");    
        }

        value.fmt(self.f);

        self.has_entries = true;

        self
    }
}

impl DebugTuple {
    pub fn finish(ref mut self) {
        self.f.print_str(")");
    }

    pub fn field<T>(ref mut self, value: T) -> Self
    where
        T: Debug 
    {
        if self.has_fields {
            self.f.print_str(", ");    
        }

        value.fmt(self.f);

        self.has_fields = true;

        self
    }
}

pub trait Debug {
    fn fmt(self, ref mut f: Formatter);
}

impl Debug for u8 {
    fn fmt(self, ref mut f: Formatter) {
        f.print_u8(self);
    }
}

impl Debug for u16 {
    fn fmt(self, ref mut f: Formatter) {
        f.print_u16(self);
    }
}

impl Debug for u32 {
    fn fmt(self, ref mut f: Formatter) {
        f.print_u32(self);
    }
}

impl Debug for u64 {
    fn fmt(self, ref mut f: Formatter) {
        f.print_u64(self);
    }
}

impl Debug for raw_ptr {
    fn fmt(self, ref mut f: Formatter) {
        let v = asm(v: self) {
            v: u64
        };
        f.print_u64(v);
    }
}

impl Debug for str {
    fn fmt(self, ref mut f: Formatter) {
        let quote = [34u8];
        f.print_str(asm(s: (__addr_of(quote), 1)) {
            s: str
        });
        f.print_str(self);
        f.print_str(asm(s: (__addr_of(quote), 1)) {
            s: str
        });
    }
}

#[test]
fn ok_print_u8() {
    let mut f = Formatter {};

    f.debug_struct("Vec")
        .field("len", 1u8)
        .field("capacity", 2u8)
        .finish();

    f.debug_list()
        .entry(1u8)
        .entry(2u8)
        .finish();

    f.debug_tuple()
        .field(1u8)
        .field("2u8")
        .finish();
}