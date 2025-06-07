library;

use ::raw_ptr::*;
use ::codec::*;
use ::debug::*;
use ::slice::*;

const STDERR: u64 = 2;

// ssize_t write(int fd, const void buf[.count], size_t count);
fn syscall_write(fd: u64, buf: raw_ptr, count: u64) {
    asm(id: 1000, fd: fd, buf: buf, count: count) {
        ecal id fd buf count;
    }
}

// int fflush(FILE *_Nullable stream);
fn syscall_fflush(fd: u64) {
    asm(id: 1001, fd: fd) {
        ecal id fd zero zero;
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

pub struct Formatter {}

impl Formatter {
    pub fn print_string_quotes(self) {
        let c = [34u8];
        syscall_write(STDERR, __addr_of(c), 1);
    }

    pub fn print_newline(self) {
        let c = [10u8];
        syscall_write(STDERR, __addr_of(c), 1);
    }

    pub fn print_str(self, s: str) {
        syscall_write(STDERR, s.as_ptr(), s.len());
    }

    pub fn print_u8(self, value: u8) {
        let mut value = value;
        let mut digits = [48u8; 64];
        let mut i = 63;
        while true {
            digits[i] = (value % 10) + 48; // ascii zero = 48
            value = value / 10;

            if value == 0 {
                break;
            }
            i -= 1;
        }

        syscall_write(STDERR, __addr_of(digits).add::<u8>(i), 64 - i);
    }

    pub fn print_u16(self, value: u16) {
        let mut value = value;
        let mut digits = [48u8; 64];
        let mut i = 63;
        while true {
            let digit = asm(v: value % 10) {
                v: u8
            };
            digits[i] = digit + 48; // ascii zero = 48
            value = value / 10;

            if value == 0 {
                break;
            }
            i -= 1;
        }

        syscall_write(STDERR, __addr_of(digits).add::<u8>(i), 64 - i);
    }

    pub fn print_u32(self, value: u32) {
        let mut value = value;
        let mut digits = [48u8; 64];
        let mut i = 63;
        while true {
            let digit = asm(v: value % 10) {
                v: u8
            };
            digits[i] = digit + 48; // ascii zero = 48
            value = value / 10;

            if value == 0 {
                break;
            }
            i -= 1;
        }

        syscall_write(STDERR, __addr_of(digits).add::<u8>(i), 64 - i);
    }

    pub fn print_u64(self, value: u64) {
        let mut value = value;
        let mut digits = [48u8; 64];
        let mut i = 63;
        while true {
            let digit = asm(v: value % 10) {
                v: u8
            };
            digits[i] = digit + 48; // ascii zero = 48
            value = value / 10;

            if value == 0 {
                break;
            }
            i -= 1;
        }

        syscall_write(STDERR, __addr_of(digits).add::<u8>(i), 64 - i);
    }

    pub fn print_u256(self, value: u256) {
        let mut value = value;
        // u256::MAX = 115792089237316195423570985008687907853269984665640564039457584007913129639935
        let mut digits = [48u8; 80];
        let mut i = 79;
        while true {
            let rem = value % 10;
            let (_, _, _, digit) = asm(rem: rem) {
                rem: (u64, u64, u64, u64)
            };
            let digit = asm(v: digit % 10) {
                v: u8
            };
            digits[i] = digit + 48; // ascii zero = 48
            value = value / 10;

            if value == 0 {
                break;
            }
            i -= 1;
        }

        syscall_write(STDERR, __addr_of(digits).add::<u8>(i), 80 - i);
    }

    pub fn print_u256_as_hex(self, value: u256, uppercase: bool) {
        let a = if uppercase {
            // ascii A
            65u8
        } else {
            // ascii a
            97u8
        };

        let mut value = value;
        // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
        let mut digits = [48u8; 66];
        digits[0] = 48; // ascii Zero
        digits[1] = 120; // ascii X
        let mut i = 65;
        while true {
            let rem = value % 16;
            let (_, _, _, digit) = asm(rem: rem) {
                rem: (u64, u64, u64, u64)
            };
            let digit = asm(v: digit % 16) {
                v: u8
            };

            if digit < 10 {
                digits[i] = digit + 48; // ascii zero = 48
            } else {
                digits[i] = (digit - 10) + a;
            }
            value = value / 16;

            if value == 0 {
                break;
            }
            i -= 1;
        }

        syscall_write(STDERR, __addr_of(digits), 66);
    }

    pub fn debug_struct(self, name: str) -> DebugStruct {
        self.print_str(name);
        self.print_str(" { ");
        DebugStruct {
            f: self,
            has_fields: false,
        }
    }

    pub fn debug_list(self) -> DebugList {
        self.print_str("[");
        DebugList {
            f: self,
            has_entries: false,
        }
    }

    pub fn debug_tuple(self, name: str) -> DebugTuple {
        if name.len() > 0 {
            self.print_str(name);
        }

        self.print_str("(");
        DebugTuple {
            f: self,
            has_fields: false,
        }
    }

    pub fn flush(self) {
        syscall_fflush(STDERR);
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
        T: Debug,
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
        T: Debug,
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
        T: Debug,
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

impl Debug for () {
    fn fmt(self, ref mut f: Formatter) {
        f.print_str("()");
    }
}

impl Debug for bool {
    fn fmt(self, ref mut f: Formatter) {
        if self {
            f.print_str("true");
        } else {
            f.print_str("false");
        }
    }
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

impl Debug for u256 {
    fn fmt(self, ref mut f: Formatter) {
        f.print_u256(self);
    }
}

impl Debug for b256 {
    fn fmt(self, ref mut f: Formatter) {
        f.print_u256_as_hex(asm(s: self) {
            s: u256
        }, true);
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
        f.print_string_quotes();
        f.print_str(self);
        f.print_string_quotes();
    }
}

impl<T> Debug for &[T]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();

        let mut i = 0;
        while i < self.len() {
            let item: T = *__elem_at(self, i);
            f = f.entry(item);
            i += 1;
        }

        f.finish();
    }
}

#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> Debug for [T; N]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();

        let mut i = 0;
        while i < N {
            f = f.entry(self[i]);
            i += 1;
        }

        f.finish();
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 0]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        f.finish();
    }
}

// BEGIN ARRAY_DEBUG
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 1]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 1 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 2]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 2 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 3]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 3 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 4]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 4 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 5]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 5 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 6]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 6 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 7]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 7 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 8]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 8 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 9]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 9 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 10]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 10 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 11]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 11 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 12]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 12 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 13]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 13 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 14]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 14 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 15]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 15 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 16]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 16 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 17]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 17 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 18]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 18 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 19]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 19 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 20]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 20 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 21]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 21 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 22]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 22 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 23]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 23 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 24]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 24 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 25]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 25 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 26]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 26 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 27]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 27 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 28]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 28 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 29]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 29 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 30]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 30 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 31]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 31 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 32]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 32 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 33]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 33 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 34]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 34 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 35]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 35 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 36]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 36 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 37]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 37 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 38]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 38 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 39]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 39 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 40]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 40 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 41]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 41 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 42]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 42 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 43]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 43 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 44]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 44 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 45]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 45 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 46]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 46 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 47]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 47 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 48]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 48 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 49]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 49 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 50]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 50 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 51]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 51 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 52]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 52 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 53]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 53 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 54]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 54 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 55]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 55 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 56]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 56 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 57]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 57 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 58]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 58 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 59]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 59 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 60]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 60 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 61]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 61 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 62]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 62 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 63]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 63 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> Debug for [T; 64]
where
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_list();
        let mut i = 0;
        while i < 64 {
            f = f.entry(self[i]);
            i += 1;
        };
        f.finish();
    }
}
// END ARRAY_DEBUG

#[cfg(experimental_const_generics = false)]
impl Debug for str[0] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}

// BEGIN STRARRAY_DEBUG
#[cfg(experimental_const_generics = false)]
impl Debug for str[1] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[2] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[3] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[4] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[5] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[6] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[7] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[8] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[9] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[10] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[11] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[12] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[13] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[14] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[15] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[16] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[17] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[18] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[19] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[20] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[21] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[22] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[23] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[24] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[25] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[26] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[27] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[28] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[29] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[30] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[31] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[32] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[33] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[34] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[35] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[36] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[37] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[38] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[39] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[40] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[41] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[42] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[43] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[44] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[45] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[46] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[47] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[48] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[49] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[50] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[51] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[52] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[53] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[54] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[55] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[56] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[57] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[58] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[59] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[60] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[61] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[62] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[63] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
#[cfg(experimental_const_generics = false)]
impl Debug for str[64] {
    fn fmt(self, ref mut f: Formatter) {
        use ::str::*;
        from_str_array(self).fmt(f);
    }
}
// END STRARRAY_DEBUG

// BEGIN TUPLES_DEBUG
impl<A> Debug for (A, )
where
    A: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        f.finish();
    }
}
impl<A, B> Debug for (A, B)
where
    A: Debug,
    B: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        f.finish();
    }
}
impl<A, B, C> Debug for (A, B, C)
where
    A: Debug,
    B: Debug,
    C: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        f.finish();
    }
}
impl<A, B, C, D> Debug for (A, B, C, D)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        f.finish();
    }
}
impl<A, B, C, D, E> Debug for (A, B, C, D, E)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        f.finish();
    }
}
impl<A, B, C, D, E, F> Debug for (A, B, C, D, E, F)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G> Debug for (A, B, C, D, E, F, G)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H> Debug for (A, B, C, D, E, F, G, H)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I> Debug for (A, B, C, D, E, F, G, H, I)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J> Debug for (A, B, C, D, E, F, G, H, I, J)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K> Debug for (A, B, C, D, E, F, G, H, I, J, K)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L> Debug for (A, B, C, D, E, F, G, H, I, J, K, L)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
    R: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        let mut f = f.field(self.17);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
    R: Debug,
    S: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        let mut f = f.field(self.17);
        let mut f = f.field(self.18);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
    R: Debug,
    S: Debug,
    T: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        let mut f = f.field(self.17);
        let mut f = f.field(self.18);
        let mut f = f.field(self.19);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
    R: Debug,
    S: Debug,
    T: Debug,
    U: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        let mut f = f.field(self.17);
        let mut f = f.field(self.18);
        let mut f = f.field(self.19);
        let mut f = f.field(self.20);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
    R: Debug,
    S: Debug,
    T: Debug,
    U: Debug,
    V: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        let mut f = f.field(self.17);
        let mut f = f.field(self.18);
        let mut f = f.field(self.19);
        let mut f = f.field(self.20);
        let mut f = f.field(self.21);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
    R: Debug,
    S: Debug,
    T: Debug,
    U: Debug,
    V: Debug,
    W: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        let mut f = f.field(self.17);
        let mut f = f.field(self.18);
        let mut f = f.field(self.19);
        let mut f = f.field(self.20);
        let mut f = f.field(self.21);
        let mut f = f.field(self.22);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
    R: Debug,
    S: Debug,
    T: Debug,
    U: Debug,
    V: Debug,
    W: Debug,
    X: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        let mut f = f.field(self.17);
        let mut f = f.field(self.18);
        let mut f = f.field(self.19);
        let mut f = f.field(self.20);
        let mut f = f.field(self.21);
        let mut f = f.field(self.22);
        let mut f = f.field(self.23);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
    R: Debug,
    S: Debug,
    T: Debug,
    U: Debug,
    V: Debug,
    W: Debug,
    X: Debug,
    Y: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        let mut f = f.field(self.17);
        let mut f = f.field(self.18);
        let mut f = f.field(self.19);
        let mut f = f.field(self.20);
        let mut f = f.field(self.21);
        let mut f = f.field(self.22);
        let mut f = f.field(self.23);
        let mut f = f.field(self.24);
        f.finish();
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z> Debug for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)
where
    A: Debug,
    B: Debug,
    C: Debug,
    D: Debug,
    E: Debug,
    F: Debug,
    G: Debug,
    H: Debug,
    I: Debug,
    J: Debug,
    K: Debug,
    L: Debug,
    M: Debug,
    N: Debug,
    O: Debug,
    P: Debug,
    Q: Debug,
    R: Debug,
    S: Debug,
    T: Debug,
    U: Debug,
    V: Debug,
    W: Debug,
    X: Debug,
    Y: Debug,
    Z: Debug,
{
    fn fmt(self, ref mut f: Formatter) {
        let mut f = f.debug_tuple("");
        let mut f = f.field(self.0);
        let mut f = f.field(self.1);
        let mut f = f.field(self.2);
        let mut f = f.field(self.3);
        let mut f = f.field(self.4);
        let mut f = f.field(self.5);
        let mut f = f.field(self.6);
        let mut f = f.field(self.7);
        let mut f = f.field(self.8);
        let mut f = f.field(self.9);
        let mut f = f.field(self.10);
        let mut f = f.field(self.11);
        let mut f = f.field(self.12);
        let mut f = f.field(self.13);
        let mut f = f.field(self.14);
        let mut f = f.field(self.15);
        let mut f = f.field(self.16);
        let mut f = f.field(self.17);
        let mut f = f.field(self.18);
        let mut f = f.field(self.19);
        let mut f = f.field(self.20);
        let mut f = f.field(self.21);
        let mut f = f.field(self.22);
        let mut f = f.field(self.23);
        let mut f = f.field(self.24);
        let mut f = f.field(self.25);
        f.finish();
    }
}
// END TUPLES_DEBUG


#[cfg(experimental_const_generics = true)]
impl<const N: u64> Debug for str[N] {
    fn fmt(self, ref mut f: Formatter) {
        //use ::str::*;
        //from_str_array(self).fmt(f);
    }
}
