script;
use core::ops::*;

/// I am a random doc comment.
struct Struct1 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct1(s: Struct1) -> u16 {
    s.field2
}

/// I am a random doc comment.
struct Struct2 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct2(s: Struct2) -> u32 {
    s.field3
}

/// I am a random doc comment.
struct Struct3 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct3(s: Struct3) -> u64 {
    s.field4
}

/// I am a random doc comment.
struct Struct4 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct4(s: Struct4) -> u8 {
    s.field1
}

/// I am a random doc comment.
struct Struct5 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct5(s: Struct5) -> u16 {
    s.field2
}

/// I am a random doc comment.
struct Struct6 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct6(s: Struct6) -> u32 {
    s.field3
}

/// I am a random doc comment.
struct Struct7 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct7(s: Struct7) -> u64 {
    s.field4
}

/// I am a random doc comment.
struct Struct8 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct8(s: Struct8) -> u8 {
    s.field1
}

/// I am a random doc comment.
struct Struct9 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct9(s: Struct9) -> u16 {
    s.field2
}

/// I am a random doc comment.
struct Struct10 {
    /// I am a random doc comment.
    field1: u8,
    /// I am a random doc comment.
    field2: u16,
    /// I am a random doc comment.
    field3: u32,
    /// I am a random doc comment.
    field4: u64,
}

/// I am a random doc comment.
fn func_struct10(s: Struct10) -> u32 {
    s.field3
}

/// I am a random doc comment.
enum Enum11 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}


/// I am a random doc comment.
fn func_enum11(e: Enum11) -> u64 {
    match e {
        Enum11::Variant1(_) => (),
        Enum11::Variant2(_) => (),
        Enum11::Variant3(_) => (),
        Enum11::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
enum Enum12 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}

/// I am a random doc comment.
fn func_enum12(e: Enum12) -> u8 {
    match e {
        Enum12::Variant1(_) => (),
        Enum12::Variant2(_) => (),
        Enum12::Variant3(_) => (),
        Enum12::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
enum Enum13 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}

/// I am a random doc comment.
fn func_enum13(e: Enum13) -> u16 {
    match e {
        Enum13::Variant1(_) => (),
        Enum13::Variant2(_) => (),
        Enum13::Variant3(_) => (),
        Enum13::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
enum Enum14 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}

/// I am a random doc comment.
fn func_enum14(e: Enum14) -> u32 {
    match e {
        Enum14::Variant1(_) => (),
        Enum14::Variant2(_) => (),
        Enum14::Variant3(_) => (),
        Enum14::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
enum Enum15 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}

/// I am a random doc comment.
fn func_enum15(e: Enum15) -> u64 {
    match e {
        Enum15::Variant1(_) => (),
        Enum15::Variant2(_) => (),
        Enum15::Variant3(_) => (),
        Enum15::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
enum Enum16 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}

/// I am a random doc comment.
fn func_enum16(e: Enum16) -> u8 {
    match e {
        Enum16::Variant1(_) => (),
        Enum16::Variant2(_) => (),
        Enum16::Variant3(_) => (),
        Enum16::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
enum Enum17 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}

/// I am a random doc comment.
fn func_enum17(e: Enum17) -> u16 {
    match e {
        Enum17::Variant1(_) => (),
        Enum17::Variant2(_) => (),
        Enum17::Variant3(_) => (),
        Enum17::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
enum Enum18 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}

/// I am a random doc comment.
fn func_enum18(e: Enum18) -> u32 {
    match e {
        Enum18::Variant1(_) => (),
        Enum18::Variant2(_) => (),
        Enum18::Variant3(_) => (),
        Enum18::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
enum Enum19 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}

/// I am a random doc comment.
fn func_enum19(e: Enum19) -> u64 {
    match e {
        Enum19::Variant1(_) => (),
        Enum19::Variant2(_) => (),
        Enum19::Variant3(_) => (),
        Enum19::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
enum Enum20 {
    /// I am a random doc comment.
    Variant1: u8,
    /// I am a random doc comment.
    Variant2: u16,
    /// I am a random doc comment.
    Variant3: u32,
    /// I am a random doc comment.
    Variant4: u64,
}

/// I am a random doc comment.
fn func_enum20(e: Enum20) -> u8 {
    match e {
        Enum20::Variant1(_) => (),
        Enum20::Variant2(_) => (),
        Enum20::Variant3(_) => (),
        Enum20::Variant4(_) => (),
    }
    0
}

/// I am a random doc comment.
fn func21(x: u64, y: u64) -> u64 {
    if x - y > 10 {
        x / y
    } else {
        y / x
    }
}

/// I am a random doc comment.
fn func22(x: u8, y: u8) -> u8 {
    if x - y > 10 {
        x / y
    } else {
        y / x
    }
}

/// I am a random doc comment.
fn func23(x: u16, y: u16) -> u16 {
    if x - y > 10 {
        x / y
    } else {
        y / x
    }
}

/// I am a random doc comment.
fn func24(x: u32, y: u32) -> u32 {
    if x - y > 10 {
        x / y
    } else {
        y / x
    }
}

/// I am a random doc comment.
fn func25(x: u64, y: u64) -> u64 {
    if x - y > 10 {
        x / y
    } else {
        y / x
    }
}

/// I am a random doc comment.
fn func26(x: u8, y: u8) -> u8 {
    if x - y > 10 {
        x / y
    } else {
        y / x
    }
}

/// I am a random doc comment.
fn func27(x: u16, y: u16) -> u16 {
    if x - y > 10 {
        x / y
    } else {
        y / x
    }
}

/// I am a random doc comment.
fn func28(x: u32, y: u32) -> u32 {
    if x - y > 10 {
        x / y
    } else {
        y / x
    }
}

/// I am a random doc comment.
fn func29(x: u16, y: u16) -> u16 {
    if x - y > 10 {
        x / y
    } else {
        y / x
    }
}


/// I am a random doc comment.
fn func30(x: u32, y: u32) -> u32 {
    if x - y > 10 {
        x + y
    } else {
        y - x
    }
}


fn main() {
    let varb_u8: u8 = 1;
    let varb_u16: u16 = 1;
    let varb_u32: u32 = 1;
    let varb_u64: u64 = 1;
    let varc_u8: u8 = 2;
    let varc_u16: u16 = 2;
    let varc_u32: u32 = 2;
    let varc_u64: u64 = 2;
    let vard_u8: u8 = 3;
    let vard_u16: u16 = 3;
    let vard_u32: u32 = 3;
    let vard_u64: u64 = 3;
    let vare_u8: u8 = 4;
    let vare_u16: u16 = 4;
    let vare_u32: u32 = 4;
    let vare_u64: u64 = 4;
    let varf_u8: u8 = 5;
    let varf_u16: u16 = 5;
    let varf_u32: u32 = 5;
    let varf_u64: u64 = 5;
    let varg_u8: u8 = 6;
    let varg_u16: u16 = 6;
    let varg_u32: u32 = 6;
    let varg_u64: u64 = 6;
    let varh_u8: u8 = 7;
    let varh_u16: u16 = 7;
    let varh_u32: u32 = 7;
    let varh_u64: u64 = 7;
    let vari_u8: u8 = 8;
    let vari_u16: u16 = 8;
    let vari_u32: u32 = 8;
    let vari_u64: u64 = 8;
    let varj_u8: u8 = 9;
    let varj_u16: u16 = 9;
    let varj_u32: u32 = 9;
    let varj_u64: u64 = 9;
    let vark_u8: u8 = 10;
    let vark_u16: u16 = 10;
    let vark_u32: u32 = 10;
    let vark_u64: u64 = 10;
    let varl_u8: u8 = 11;
    let varl_u16: u16 = 11;
    let varl_u32: u32 = 11;
    let varl_u64: u64 = 11;
    let varm_u8: u8 = 12;
    let varm_u16: u16 = 12;
    let varm_u32: u32 = 12;
    let varm_u64: u64 = 12;
    let varn_u8: u8 = 13;
    let varn_u16: u16 = 13;
    let varn_u32: u32 = 13;
    let varn_u64: u64 = 13;
    let varo_u8: u8 = 14;
    let varo_u16: u16 = 14;
    let varo_u32: u32 = 14;
    let varo_u64: u64 = 14;
    let varp_u8: u8 = 15;
    let varp_u16: u16 = 15;
    let varp_u32: u32 = 15;
    let varp_u64: u64 = 15;
    let varq_u8: u8 = 16;
    let varq_u16: u16 = 16;
    let varq_u32: u32 = 16;
    let varq_u64: u64 = 16;
    let varr_u8: u8 = 17;
    let varr_u16: u16 = 17;
    let varr_u32: u32 = 17;
    let varr_u64: u64 = 17;
    let vars_u8: u8 = 18;
    let vars_u16: u16 = 18;
    let vars_u32: u32 = 18;
    let vars_u64: u64 = 18;
    let vart_u8: u8 = 19;
    let vart_u16: u16 = 19;
    let vart_u32: u32 = 19;
    let vart_u64: u64 = 19;
    let varu_u8: u8 = 20;
    let varu_u16: u16 = 20;
    let varu_u32: u32 = 20;
    let varu_u64: u64 = 20;
    let varv_u8: u8 = 21;
    let varv_u16: u16 = 21;
    let varv_u32: u32 = 21;
    let varv_u64: u64 = 21;
    let varw_u8: u8 = 22;
    let varw_u16: u16 = 22;
    let varw_u32: u32 = 22;
    let varw_u64: u64 = 22;
    let varx_u8: u8 = 23;
    let varx_u16: u16 = 23;
    let varx_u32: u32 = 23;
    let varx_u64: u64 = 23;
    let vary_u8: u8 = 24;
    let vary_u16: u16 = 24;
    let vary_u32: u32 = 24;
    let vary_u64: u64 = 24;
    let varz_u8: u8 = 25;
    let varz_u16: u16 = 25;
    let varz_u32: u32 = 25;
    let varz_u64: u64 = 25;
    let vara_u8: u8 = 26;
    let vara_u16: u16 = 26;
    let vara_u32: u32 = 26;
    let vara_u64: u64 = 26;
    let varb_u8: u8 = 27;
    let varb_u16: u16 = 27;
    let varb_u32: u32 = 27;
    let varb_u64: u64 = 27;
    let varc_u8: u8 = 28;
    let varc_u16: u16 = 28;
    let varc_u32: u32 = 28;
    let varc_u64: u64 = 28;
    let vard_u8: u8 = 29;
    let vard_u16: u16 = 29;
    let vard_u32: u32 = 29;
    let vard_u64: u64 = 29;
    let vare_u8: u8 = 30;
    let vare_u16: u16 = 30;
    let vare_u32: u32 = 30;
    let vare_u64: u64 = 30;
    let varf_u8: u8 = 31;
    let varf_u16: u16 = 31;
    let varf_u32: u32 = 31;
    let varf_u64: u64 = 31;
    let varg_u8: u8 = 32;
    let varg_u16: u16 = 32;
    let varg_u32: u32 = 32;
    let varg_u64: u64 = 32;
    let varh_u8: u8 = 33;
    let varh_u16: u16 = 33;
    let varh_u32: u32 = 33;
    let varh_u64: u64 = 33;
    let vari_u8: u8 = 34;
    let vari_u16: u16 = 34;
    let vari_u32: u32 = 34;
    let vari_u64: u64 = 34;
    let varj_u8: u8 = 35;
    let varj_u16: u16 = 35;
    let varj_u32: u32 = 35;
    let varj_u64: u64 = 35;
    let vark_u8: u8 = 36;
    let vark_u16: u16 = 36;
    let vark_u32: u32 = 36;
    let vark_u64: u64 = 36;
    let varl_u8: u8 = 37;
    let varl_u16: u16 = 37;
    let varl_u32: u32 = 37;
    let varl_u64: u64 = 37;
    let varm_u8: u8 = 38;
    let varm_u16: u16 = 38;
    let varm_u32: u32 = 38;
    let varm_u64: u64 = 38;
    let varn_u8: u8 = 39;
    let varn_u16: u16 = 39;
    let varn_u32: u32 = 39;
    let varn_u64: u64 = 39;
    let varo_u8: u8 = 40;
    let varo_u16: u16 = 40;
    let varo_u32: u32 = 40;
    let varo_u64: u64 = 40;
    let varp_u8: u8 = 41;
    let varp_u16: u16 = 41;
    let varp_u32: u32 = 41;
    let varp_u64: u64 = 41;
    let varq_u8: u8 = 42;
    let varq_u16: u16 = 42;
    let varq_u32: u32 = 42;
    let varq_u64: u64 = 42;
    let varr_u8: u8 = 43;
    let varr_u16: u16 = 43;
    let varr_u32: u32 = 43;
    let varr_u64: u64 = 43;
    let vars_u8: u8 = 44;
    let vars_u16: u16 = 44;
    let vars_u32: u32 = 44;
    let vars_u64: u64 = 44;
    let vart_u8: u8 = 45;
    let vart_u16: u16 = 45;
    let vart_u32: u32 = 45;
    let vart_u64: u64 = 45;
    let varu_u8: u8 = 46;
    let varu_u16: u16 = 46;
    let varu_u32: u32 = 46;
    let varu_u64: u64 = 46;
    let varv_u8: u8 = 47;
    let varv_u16: u16 = 47;
    let varv_u32: u32 = 47;
    let varv_u64: u64 = 47;
    let varw_u8: u8 = 48;
    let varw_u16: u16 = 48;
    let varw_u32: u32 = 48;
    let varw_u64: u64 = 48;
    let varx_u8: u8 = 49;
    let varx_u16: u16 = 49;
    let varx_u32: u32 = 49;
    let varx_u64: u64 = 49;
    let vary_u8: u8 = 50;
    let vary_u16: u16 = 50;
    let vary_u32: u32 = 50;
    let vary_u64: u64 = 50;
    let varz_u8: u8 = 51;
    let varz_u16: u16 = 51;
    let varz_u32: u32 = 51;
    let varz_u64: u64 = 51;
    let vara_u8: u8 = 52;
    let vara_u16: u16 = 52;
    let vara_u32: u32 = 52;
    let vara_u64: u64 = 52;
    let varb_u8: u8 = 53;
    let varb_u16: u16 = 53;
    let varb_u32: u32 = 53;
    let varb_u64: u64 = 53;
    let varc_u8: u8 = 54;
    let varc_u16: u16 = 54;
    let varc_u32: u32 = 54;
    let varc_u64: u64 = 54;
    let vard_u8: u8 = 55;
    let vard_u16: u16 = 55;
    let vard_u32: u32 = 55;
    let vard_u64: u64 = 55;
    let vare_u8: u8 = 56;
    let vare_u16: u16 = 56;
    let vare_u32: u32 = 56;
    let vare_u64: u64 = 56;
    let varf_u8: u8 = 57;
    let varf_u16: u16 = 57;
    let varf_u32: u32 = 57;
    let varf_u64: u64 = 57;
    let varg_u8: u8 = 58;
    let varg_u16: u16 = 58;
    let varg_u32: u32 = 58;
    let varg_u64: u64 = 58;
    let varh_u8: u8 = 59;
    let varh_u16: u16 = 59;
    let varh_u32: u32 = 59;
    let varh_u64: u64 = 59;
    let vari_u8: u8 = 60;
    let vari_u16: u16 = 60;
    let vari_u32: u32 = 60;
    let vari_u64: u64 = 60;
    let varj_u8: u8 = 61;
    let varj_u16: u16 = 61;
    let varj_u32: u32 = 61;
    let varj_u64: u64 = 61;
    let vark_u8: u8 = 62;
    let vark_u16: u16 = 62;
    let vark_u32: u32 = 62;
    let vark_u64: u64 = 62;
    let varl_u8: u8 = 63;
    let varl_u16: u16 = 63;
    let varl_u32: u32 = 63;
    let varl_u64: u64 = 63;
    let varm_u8: u8 = 64;
    let varm_u16: u16 = 64;
    let varm_u32: u32 = 64;
    let varm_u64: u64 = 64;
    let varn_u8: u8 = 65;
    let varn_u16: u16 = 65;
    let varn_u32: u32 = 65;
    let varn_u64: u64 = 65;
    let varo_u8: u8 = 66;
    let varo_u16: u16 = 66;
    let varo_u32: u32 = 66;
    let varo_u64: u64 = 66;
    let varp_u8: u8 = 67;
    let varp_u16: u16 = 67;
    let varp_u32: u32 = 67;
    let varp_u64: u64 = 67;
    let varq_u8: u8 = 68;
    let varq_u16: u16 = 68;
    let varq_u32: u32 = 68;
    let varq_u64: u64 = 68;
    let varr_u8: u8 = 69;
    let varr_u16: u16 = 69;
    let varr_u32: u32 = 69;
    let varr_u64: u64 = 69;
    let vars_u8: u8 = 70;
    let vars_u16: u16 = 70;
    let vars_u32: u32 = 70;
    let vars_u64: u64 = 70;
    let vart_u8: u8 = 71;
    let vart_u16: u16 = 71;
    let vart_u32: u32 = 71;
    let vart_u64: u64 = 71;
    let varu_u8: u8 = 72;
    let varu_u16: u16 = 72;
    let varu_u32: u32 = 72;
    let varu_u64: u64 = 72;
    let varv_u8: u8 = 73;
    let varv_u16: u16 = 73;
    let varv_u32: u32 = 73;
    let varv_u64: u64 = 73;
    let varw_u8: u8 = 74;
    let varw_u16: u16 = 74;
    let varw_u32: u32 = 74;
    let varw_u64: u64 = 74;
    let varx_u8: u8 = 75;
    let varx_u16: u16 = 75;
    let varx_u32: u32 = 75;
    let varx_u64: u64 = 75;
    let vary_u8: u8 = 76;
    let vary_u16: u16 = 76;
    let vary_u32: u32 = 76;
    let vary_u64: u64 = 76;
    let varz_u8: u8 = 77;
    let varz_u16: u16 = 77;
    let varz_u32: u32 = 77;
    let varz_u64: u64 = 77;
    let vara_u8: u8 = 78;
    let vara_u16: u16 = 78;
    let vara_u32: u32 = 78;
    let vara_u64: u64 = 78;
    let varb_u8: u8 = 79;
    let varb_u16: u16 = 79;
    let varb_u32: u32 = 79;
    let varb_u64: u64 = 79;
    let varc_u8: u8 = 80;
    let varc_u16: u16 = 80;
    let varc_u32: u32 = 80;
    let varc_u64: u64 = 80;
    let vard_u8: u8 = 81;
    let vard_u16: u16 = 81;
    let vard_u32: u32 = 81;
    let vard_u64: u64 = 81;
    let vare_u8: u8 = 82;
    let vare_u16: u16 = 82;
    let vare_u32: u32 = 82;
    let vare_u64: u64 = 82;
    let varf_u8: u8 = 83;
    let varf_u16: u16 = 83;
    let varf_u32: u32 = 83;
    let varf_u64: u64 = 83;
    let varg_u8: u8 = 84;
    let varg_u16: u16 = 84;
    let varg_u32: u32 = 84;
    let varg_u64: u64 = 84;
    let varh_u8: u8 = 85;
    let varh_u16: u16 = 85;
    let varh_u32: u32 = 85;
    let varh_u64: u64 = 85;
    let vari_u8: u8 = 86;
    let vari_u16: u16 = 86;
    let vari_u32: u32 = 86;
    let vari_u64: u64 = 86;
    let varj_u8: u8 = 87;
    let varj_u16: u16 = 87;
    let varj_u32: u32 = 87;
    let varj_u64: u64 = 87;
    let vark_u8: u8 = 88;
    let vark_u16: u16 = 88;
    let vark_u32: u32 = 88;
    let vark_u64: u64 = 88;
    let varl_u8: u8 = 89;
    let varl_u16: u16 = 89;
    let varl_u32: u32 = 89;
    let varl_u64: u64 = 89;
    let varm_u8: u8 = 90;
    let varm_u16: u16 = 90;
    let varm_u32: u32 = 90;
    let varm_u64: u64 = 90;
    let varn_u8: u8 = 91;
    let varn_u16: u16 = 91;
    let varn_u32: u32 = 91;
    let varn_u64: u64 = 91;
    let varo_u8: u8 = 92;
    let varo_u16: u16 = 92;
    let varo_u32: u32 = 92;
    let varo_u64: u64 = 92;
    let varp_u8: u8 = 93;
    let varp_u16: u16 = 93;
    let varp_u32: u32 = 93;
    let varp_u64: u64 = 93;
    let varq_u8: u8 = 94;
    let varq_u16: u16 = 94;
    let varq_u32: u32 = 94;
    let varq_u64: u64 = 94;
    let varr_u8: u8 = 95;
    let varr_u16: u16 = 95;
    let varr_u32: u32 = 95;
    let varr_u64: u64 = 95;
    let vars_u8: u8 = 96;
    let vars_u16: u16 = 96;
    let vars_u32: u32 = 96;
    let vars_u64: u64 = 96;
    let vart_u8: u8 = 97;
    let vart_u16: u16 = 97;
    let vart_u32: u32 = 97;
    let vart_u64: u64 = 97;
    let varu_u8: u8 = 98;
    let varu_u16: u16 = 98;
    let varu_u32: u32 = 98;
    let varu_u64: u64 = 98;
    let varv_u8: u8 = 99;
    let varv_u16: u16 = 99;
    let varv_u32: u32 = 99;
    let varv_u64: u64 = 99;
    let varw_u8: u8 = 100;
    let varw_u16: u16 = 100;
    let varw_u32: u32 = 100;
    let varw_u64: u64 = 100;
    let varx_u8: u8 = 101;
    let varx_u16: u16 = 101;
    let varx_u32: u32 = 101;
    let varx_u64: u64 = 101;
    let vary_u8: u8 = 102;
    let vary_u16: u16 = 102;
    let vary_u32: u32 = 102;
    let vary_u64: u64 = 102;
    let varz_u8: u8 = 103;
    let varz_u16: u16 = 103;
    let varz_u32: u32 = 103;
    let varz_u64: u64 = 103;
    let vara_u8: u8 = 104;
    let vara_u16: u16 = 104;
    let vara_u32: u32 = 104;
    let vara_u64: u64 = 104;
    let varb_u8: u8 = 105;
    let varb_u16: u16 = 105;
    let varb_u32: u32 = 105;
    let varb_u64: u64 = 105;
    let varc_u8: u8 = 106;
    let varc_u16: u16 = 106;
    let varc_u32: u32 = 106;
    let varc_u64: u64 = 106;
    let vard_u8: u8 = 107;
    let vard_u16: u16 = 107;
    let vard_u32: u32 = 107;
    let vard_u64: u64 = 107;
    let vare_u8: u8 = 108;
    let vare_u16: u16 = 108;
    let vare_u32: u32 = 108;
    let vare_u64: u64 = 108;
    let varf_u8: u8 = 109;
    let varf_u16: u16 = 109;
    let varf_u32: u32 = 109;
    let varf_u64: u64 = 109;
    let varg_u8: u8 = 110;
    let varg_u16: u16 = 110;
    let varg_u32: u32 = 110;
    let varg_u64: u64 = 110;
    let varh_u8: u8 = 111;
    let varh_u16: u16 = 111;
    let varh_u32: u32 = 111;
    let varh_u64: u64 = 111;
    let vari_u8: u8 = 112;
    let vari_u16: u16 = 112;
    let vari_u32: u32 = 112;
    let vari_u64: u64 = 112;
    let varj_u8: u8 = 113;
    let varj_u16: u16 = 113;
    let varj_u32: u32 = 113;
    let varj_u64: u64 = 113;
    let vark_u8: u8 = 114;
    let vark_u16: u16 = 114;
    let vark_u32: u32 = 114;
    let vark_u64: u64 = 114;
    let varl_u8: u8 = 115;
    let varl_u16: u16 = 115;
    let varl_u32: u32 = 115;
    let varl_u64: u64 = 115;
    let varm_u8: u8 = 116;
    let varm_u16: u16 = 116;
    let varm_u32: u32 = 116;
    let varm_u64: u64 = 116;
    let varn_u8: u8 = 117;
    let varn_u16: u16 = 117;
    let varn_u32: u32 = 117;
    let varn_u64: u64 = 117;
    let varo_u8: u8 = 118;
    let varo_u16: u16 = 118;
    let varo_u32: u32 = 118;
    let varo_u64: u64 = 118;
    let varp_u8: u8 = 119;
    let varp_u16: u16 = 119;
    let varp_u32: u32 = 119;
    let varp_u64: u64 = 119;
    let varq_u8: u8 = 120;
    let varq_u16: u16 = 120;
    let varq_u32: u32 = 120;
    let varq_u64: u64 = 120;
    let varr_u8: u8 = 121;
    let varr_u16: u16 = 121;
    let varr_u32: u32 = 121;
    let varr_u64: u64 = 121;
    let vars_u8: u8 = 122;
    let vars_u16: u16 = 122;
    let vars_u32: u32 = 122;
    let vars_u64: u64 = 122;
    let vart_u8: u8 = 123;
    let vart_u16: u16 = 123;
    let vart_u32: u32 = 123;
    let vart_u64: u64 = 123;
    let varu_u8: u8 = 124;
    let varu_u16: u16 = 124;
    let varu_u32: u32 = 124;
    let varu_u64: u64 = 124;
    let varv_u8: u8 = 125;
    let varv_u16: u16 = 125;
    let varv_u32: u32 = 125;
    let varv_u64: u64 = 125;
    let varw_u8: u8 = 126;
    let varw_u16: u16 = 126;
    let varw_u32: u32 = 126;
    let varw_u64: u64 = 126;
    let varx_u8: u8 = 127;
    let varx_u16: u16 = 127;
    let varx_u32: u32 = 127;
    let varx_u64: u64 = 127;
    let vary_u8: u8 = 128;
    let vary_u16: u16 = 128;
    let vary_u32: u32 = 128;
    let vary_u64: u64 = 128;
    let varz_u8: u8 = 129;
    let varz_u16: u16 = 129;
    let varz_u32: u32 = 129;
    let varz_u64: u64 = 129;
    let vara_u8: u8 = 130;
    let vara_u16: u16 = 130;
    let vara_u32: u32 = 130;
    let vara_u64: u64 = 130;
    let varb_u8: u8 = 131;
    let varb_u16: u16 = 131;
    let varb_u32: u32 = 131;
    let varb_u64: u64 = 131;
    let varc_u8: u8 = 132;
    let varc_u16: u16 = 132;
    let varc_u32: u32 = 132;
    let varc_u64: u64 = 132;
    let vard_u8: u8 = 133;
    let vard_u16: u16 = 133;
    let vard_u32: u32 = 133;
    let vard_u64: u64 = 133;
    let vare_u8: u8 = 134;
    let vare_u16: u16 = 134;
    let vare_u32: u32 = 134;
    let vare_u64: u64 = 134;
    let varf_u8: u8 = 135;
    let varf_u16: u16 = 135;
    let varf_u32: u32 = 135;
    let varf_u64: u64 = 135;
    let varg_u8: u8 = 136;
    let varg_u16: u16 = 136;
    let varg_u32: u32 = 136;
    let varg_u64: u64 = 136;
    let varh_u8: u8 = 137;
    let varh_u16: u16 = 137;
    let varh_u32: u32 = 137;
    let varh_u64: u64 = 137;
    let vari_u8: u8 = 138;
    let vari_u16: u16 = 138;
    let vari_u32: u32 = 138;
    let vari_u64: u64 = 138;
    let varj_u8: u8 = 139;
    let varj_u16: u16 = 139;
    let varj_u32: u32 = 139;
    let varj_u64: u64 = 139;
    let vark_u8: u8 = 140;
    let vark_u16: u16 = 140;
    let vark_u32: u32 = 140;
    let vark_u64: u64 = 140;
    let varl_u8: u8 = 141;
    let varl_u16: u16 = 141;
    let varl_u32: u32 = 141;
    let varl_u64: u64 = 141;
    let varm_u8: u8 = 142;
    let varm_u16: u16 = 142;
    let varm_u32: u32 = 142;
    let varm_u64: u64 = 142;
    let varn_u8: u8 = 143;
    let varn_u16: u16 = 143;
    let varn_u32: u32 = 143;
    let varn_u64: u64 = 143;
    let varo_u8: u8 = 144;
    let varo_u16: u16 = 144;
    let varo_u32: u32 = 144;
    let varo_u64: u64 = 144;
    let varp_u8: u8 = 145;
    let varp_u16: u16 = 145;
    let varp_u32: u32 = 145;
    let varp_u64: u64 = 145;
    let varq_u8: u8 = 146;
    let varq_u16: u16 = 146;
    let varq_u32: u32 = 146;
    let varq_u64: u64 = 146;
    let varr_u8: u8 = 147;
    let varr_u16: u16 = 147;
    let varr_u32: u32 = 147;
    let varr_u64: u64 = 147;
    let vars_u8: u8 = 148;
    let vars_u16: u16 = 148;
    let vars_u32: u32 = 148;
    let vars_u64: u64 = 148;
    let vart_u8: u8 = 149;
    let vart_u16: u16 = 149;
    let vart_u32: u32 = 149;
    let vart_u64: u64 = 149;
    let varu_u8: u8 = 150;
    let varu_u16: u16 = 150;
    let varu_u32: u32 = 150;
    let varu_u64: u64 = 150;
    let varv_u8: u8 = 151;
    let varv_u16: u16 = 151;
    let varv_u32: u32 = 151;
    let varv_u64: u64 = 151;
    let varw_u8: u8 = 152;
    let varw_u16: u16 = 152;
    let varw_u32: u32 = 152;
    let varw_u64: u64 = 152;
    let varx_u8: u8 = 153;
    let varx_u16: u16 = 153;
    let varx_u32: u32 = 153;
    let varx_u64: u64 = 153;
    let vary_u8: u8 = 154;
    let vary_u16: u16 = 154;
    let vary_u32: u32 = 154;
    let vary_u64: u64 = 154;
    let varz_u8: u8 = 155;
    let varz_u16: u16 = 155;
    let varz_u32: u32 = 155;
    let varz_u64: u64 = 155;
    let vara_u8: u8 = 156;
    let vara_u16: u16 = 156;
    let vara_u32: u32 = 156;
    let vara_u64: u64 = 156;
    let varb_u8: u8 = 157;
    let varb_u16: u16 = 157;
    let varb_u32: u32 = 157;
    let varb_u64: u64 = 157;
    let varc_u8: u8 = 158;
    let varc_u16: u16 = 158;
    let varc_u32: u32 = 158;
    let varc_u64: u64 = 158;
    let vard_u8: u8 = 159;
    let vard_u16: u16 = 159;
    let vard_u32: u32 = 159;
    let vard_u64: u64 = 159;
    let vare_u8: u8 = 160;
    let vare_u16: u16 = 160;
    let vare_u32: u32 = 160;
    let vare_u64: u64 = 160;
    let varf_u8: u8 = 161;
    let varf_u16: u16 = 161;
    let varf_u32: u32 = 161;
    let varf_u64: u64 = 161;
    let varg_u8: u8 = 162;
    let varg_u16: u16 = 162;
    let varg_u32: u32 = 162;
    let varg_u64: u64 = 162;
    let varh_u8: u8 = 163;
    let varh_u16: u16 = 163;
    let varh_u32: u32 = 163;
    let varh_u64: u64 = 163;
    let vari_u8: u8 = 164;
    let vari_u16: u16 = 164;
    let vari_u32: u32 = 164;
    let vari_u64: u64 = 164;
    let varj_u8: u8 = 165;
    let varj_u16: u16 = 165;
    let varj_u32: u32 = 165;
    let varj_u64: u64 = 165;
    let vark_u8: u8 = 166;
    let vark_u16: u16 = 166;
    let vark_u32: u32 = 166;
    let vark_u64: u64 = 166;
    let varl_u8: u8 = 167;
    let varl_u16: u16 = 167;
    let varl_u32: u32 = 167;
    let varl_u64: u64 = 167;
    let varm_u8: u8 = 168;
    let varm_u16: u16 = 168;
    let varm_u32: u32 = 168;
    let varm_u64: u64 = 168;
    let varn_u8: u8 = 169;
    let varn_u16: u16 = 169;
    let varn_u32: u32 = 169;
    let varn_u64: u64 = 169;
    let varo_u8: u8 = 170;
    let varo_u16: u16 = 170;
    let varo_u32: u32 = 170;
    let varo_u64: u64 = 170;
    let varp_u8: u8 = 171;
    let varp_u16: u16 = 171;
    let varp_u32: u32 = 171;
    let varp_u64: u64 = 171;
    let varq_u8: u8 = 172;
    let varq_u16: u16 = 172;
    let varq_u32: u32 = 172;
    let varq_u64: u64 = 172;
    let varr_u8: u8 = 173;
    let varr_u16: u16 = 173;
    let varr_u32: u32 = 173;
    let varr_u64: u64 = 173;
    let vars_u8: u8 = 174;
    let vars_u16: u16 = 174;
    let vars_u32: u32 = 174;
    let vars_u64: u64 = 174;
    let vart_u8: u8 = 175;
    let vart_u16: u16 = 175;
    let vart_u32: u32 = 175;
    let vart_u64: u64 = 175;
    let varu_u8: u8 = 176;
    let varu_u16: u16 = 176;
    let varu_u32: u32 = 176;
    let varu_u64: u64 = 176;
    let varv_u8: u8 = 177;
    let varv_u16: u16 = 177;
    let varv_u32: u32 = 177;
    let varv_u64: u64 = 177;
    let varw_u8: u8 = 178;
    let varw_u16: u16 = 178;
    let varw_u32: u32 = 178;
    let varw_u64: u64 = 178;
    let varx_u8: u8 = 179;
    let varx_u16: u16 = 179;
    let varx_u32: u32 = 179;
    let varx_u64: u64 = 179;
    let vary_u8: u8 = 180;
    let vary_u16: u16 = 180;
    let vary_u32: u32 = 180;
    let vary_u64: u64 = 180;
    let varz_u8: u8 = 181;
    let varz_u16: u16 = 181;
    let varz_u32: u32 = 181;
    let varz_u64: u64 = 181;
    let vara_u8: u8 = 182;
    let vara_u16: u16 = 182;
    let vara_u32: u32 = 182;
    let vara_u64: u64 = 182;
    let varb_u8: u8 = 183;
    let varb_u16: u16 = 183;
    let varb_u32: u32 = 183;
    let varb_u64: u64 = 183;
    let varc_u8: u8 = 184;
    let varc_u16: u16 = 184;
    let varc_u32: u32 = 184;
    let varc_u64: u64 = 184;
    let vard_u8: u8 = 185;
    let vard_u16: u16 = 185;
    let vard_u32: u32 = 185;
    let vard_u64: u64 = 185;
    let vare_u8: u8 = 186;
    let vare_u16: u16 = 186;
    let vare_u32: u32 = 186;
    let vare_u64: u64 = 186;
    let varf_u8: u8 = 187;
    let varf_u16: u16 = 187;
    let varf_u32: u32 = 187;
    let varf_u64: u64 = 187;
    let varg_u8: u8 = 188;
    let varg_u16: u16 = 188;
    let varg_u32: u32 = 188;
    let varg_u64: u64 = 188;
    let varh_u8: u8 = 189;
    let varh_u16: u16 = 189;
    let varh_u32: u32 = 189;
    let varh_u64: u64 = 189;
    let vari_u8: u8 = 190;
    let vari_u16: u16 = 190;
    let vari_u32: u32 = 190;
    let vari_u64: u64 = 190;
    let varj_u8: u8 = 191;
    let varj_u16: u16 = 191;
    let varj_u32: u32 = 191;
    let varj_u64: u64 = 191;
    let vark_u8: u8 = 192;
    let vark_u16: u16 = 192;
    let vark_u32: u32 = 192;
    let vark_u64: u64 = 192;
    let varl_u8: u8 = 193;
    let varl_u16: u16 = 193;
    let varl_u32: u32 = 193;
    let varl_u64: u64 = 193;
    let varm_u8: u8 = 194;
    let varm_u16: u16 = 194;
    let varm_u32: u32 = 194;
    let varm_u64: u64 = 194;
    let varn_u8: u8 = 195;
    let varn_u16: u16 = 195;
    let varn_u32: u32 = 195;
    let varn_u64: u64 = 195;
    let varo_u8: u8 = 196;
    let varo_u16: u16 = 196;
    let varo_u32: u32 = 196;
    let varo_u64: u64 = 196;
    let varp_u8: u8 = 197;
    let varp_u16: u16 = 197;
    let varp_u32: u32 = 197;
    let varp_u64: u64 = 197;
    let varq_u8: u8 = 198;
    let varq_u16: u16 = 198;
    let varq_u32: u32 = 198;
    let varq_u64: u64 = 198;
    let varr_u8: u8 = 199;
    let varr_u16: u16 = 199;
    let varr_u32: u32 = 199;
    let varr_u64: u64 = 199;
    let vars_u8: u8 = 200;
    let vars_u16: u16 = 200;
    let vars_u32: u32 = 200;
    let vars_u64: u64 = 200;
    let vart_u8: u8 = 201;
    let vart_u16: u16 = 201;
    let vart_u32: u32 = 201;
    let vart_u64: u64 = 201;
    let varu_u8: u8 = 202;
    let varu_u16: u16 = 202;
    let varu_u32: u32 = 202;
    let varu_u64: u64 = 202;
    let varv_u8: u8 = 203;
    let varv_u16: u16 = 203;
    let varv_u32: u32 = 203;
    let varv_u64: u64 = 203;
    let varw_u8: u8 = 204;
    let varw_u16: u16 = 204;
    let varw_u32: u32 = 204;
    let varw_u64: u64 = 204;
    let varx_u8: u8 = 205;
    let varx_u16: u16 = 205;
    let varx_u32: u32 = 205;
    let varx_u64: u64 = 205;
    let vary_u8: u8 = 206;
    let vary_u16: u16 = 206;
    let vary_u32: u32 = 206;
    let vary_u64: u64 = 206;
    let varz_u8: u8 = 207;
    let varz_u16: u16 = 207;
    let varz_u32: u32 = 207;
    let varz_u64: u64 = 207;
    let vara_u8: u8 = 208;
    let vara_u16: u16 = 208;
    let vara_u32: u32 = 208;
    let vara_u64: u64 = 208;
    let varb_u8: u8 = 209;
    let varb_u16: u16 = 209;
    let varb_u32: u32 = 209;
    let varb_u64: u64 = 209;
    let varc_u8: u8 = 210;
    let varc_u16: u16 = 210;
    let varc_u32: u32 = 210;
    let varc_u64: u64 = 210;
    let vard_u8: u8 = 211;
    let vard_u16: u16 = 211;
    let vard_u32: u32 = 211;
    let vard_u64: u64 = 211;
    let vare_u8: u8 = 212;
    let vare_u16: u16 = 212;
    let vare_u32: u32 = 212;
    let vare_u64: u64 = 212;
    let varf_u8: u8 = 213;
    let varf_u16: u16 = 213;
    let varf_u32: u32 = 213;
    let varf_u64: u64 = 213;
    let varg_u8: u8 = 214;
    let varg_u16: u16 = 214;
    let varg_u32: u32 = 214;
    let varg_u64: u64 = 214;
    let varh_u8: u8 = 215;
    let varh_u16: u16 = 215;
    let varh_u32: u32 = 215;
    let varh_u64: u64 = 215;
    let vari_u8: u8 = 216;
    let vari_u16: u16 = 216;
    let vari_u32: u32 = 216;
    let vari_u64: u64 = 216;
    let varj_u8: u8 = 217;
    let varj_u16: u16 = 217;
    let varj_u32: u32 = 217;
    let varj_u64: u64 = 217;
    let vark_u8: u8 = 218;
    let vark_u16: u16 = 218;
    let vark_u32: u32 = 218;
    let vark_u64: u64 = 218;
    let varl_u8: u8 = 219;
    let varl_u16: u16 = 219;
    let varl_u32: u32 = 219;
    let varl_u64: u64 = 219;
    let varm_u8: u8 = 220;
    let varm_u16: u16 = 220;
    let varm_u32: u32 = 220;
    let varm_u64: u64 = 220;
    let varn_u8: u8 = 221;
    let varn_u16: u16 = 221;
    let varn_u32: u32 = 221;
    let varn_u64: u64 = 221;
    let varo_u8: u8 = 222;
    let varo_u16: u16 = 222;
    let varo_u32: u32 = 222;
    let varo_u64: u64 = 222;
    let varp_u8: u8 = 223;
    let varp_u16: u16 = 223;
    let varp_u32: u32 = 223;
    let varp_u64: u64 = 223;
    let varq_u8: u8 = 224;
    let varq_u16: u16 = 224;
    let varq_u32: u32 = 224;
    let varq_u64: u64 = 224;
    let varr_u8: u8 = 225;
    let varr_u16: u16 = 225;
    let varr_u32: u32 = 225;
    let varr_u64: u64 = 225;
    let vars_u8: u8 = 226;
    let vars_u16: u16 = 226;
    let vars_u32: u32 = 226;
    let vars_u64: u64 = 226;
    let vart_u8: u8 = 227;
    let vart_u16: u16 = 227;
    let vart_u32: u32 = 227;
    let vart_u64: u64 = 227;
    let varu_u8: u8 = 228;
    let varu_u16: u16 = 228;
    let varu_u32: u32 = 228;
    let varu_u64: u64 = 228;
    let varv_u8: u8 = 229;
    let varv_u16: u16 = 229;
    let varv_u32: u32 = 229;
    let varv_u64: u64 = 229;
    let varw_u8: u8 = 230;
    let varw_u16: u16 = 230;
    let varw_u32: u32 = 230;
    let varw_u64: u64 = 230;
    let varx_u8: u8 = 231;
    let varx_u16: u16 = 231;
    let varx_u32: u32 = 231;
    let varx_u64: u64 = 231;
    let vary_u8: u8 = 232;
    let vary_u16: u16 = 232;
    let vary_u32: u32 = 232;
    let vary_u64: u64 = 232;
    let varz_u8: u8 = 233;
    let varz_u16: u16 = 233;
    let varz_u32: u32 = 233;
    let varz_u64: u64 = 233;
    let vara_u8: u8 = 234;
    let vara_u16: u16 = 234;
    let vara_u32: u32 = 234;
    let vara_u64: u64 = 234;
    let varb_u8: u8 = 235;
    let varb_u16: u16 = 235;
    let varb_u32: u32 = 235;
    let varb_u64: u64 = 235;
    let varc_u8: u8 = 236;
    let varc_u16: u16 = 236;
    let varc_u32: u32 = 236;
    let varc_u64: u64 = 236;
    let vard_u8: u8 = 237;
    let vard_u16: u16 = 237;
    let vard_u32: u32 = 237;
    let vard_u64: u64 = 237;
    let vare_u8: u8 = 238;
    let vare_u16: u16 = 238;
    let vare_u32: u32 = 238;
    let vare_u64: u64 = 238;
    let varf_u8: u8 = 239;
    let varf_u16: u16 = 239;
    let varf_u32: u32 = 239;
    let varf_u64: u64 = 239;
    let varg_u8: u8 = 240;
    let varg_u16: u16 = 240;
    let varg_u32: u32 = 240;
    let varg_u64: u64 = 240;
    let varh_u8: u8 = 241;
    let varh_u16: u16 = 241;
    let varh_u32: u32 = 241;
    let varh_u64: u64 = 241;
    let vari_u8: u8 = 242;
    let vari_u16: u16 = 242;
    let vari_u32: u32 = 242;
    let vari_u64: u64 = 242;
    let varj_u8: u8 = 243;
    let varj_u16: u16 = 243;
    let varj_u32: u32 = 243;
    let varj_u64: u64 = 243;
    let vark_u8: u8 = 244;
    let vark_u16: u16 = 244;
    let vark_u32: u32 = 244;
    let vark_u64: u64 = 244;
    let varl_u8: u8 = 245;
    let varl_u16: u16 = 245;
    let varl_u32: u32 = 245;
    let varl_u64: u64 = 245;
    let varm_u8: u8 = 246;
    let varm_u16: u16 = 246;
    let varm_u32: u32 = 246;
    let varm_u64: u64 = 246;
    let varn_u8: u8 = 247;
    let varn_u16: u16 = 247;
    let varn_u32: u32 = 247;
    let varn_u64: u64 = 247;
    let varo_u8: u8 = 248;
    let varo_u16: u16 = 248;
    let varo_u32: u32 = 248;
    let varo_u64: u64 = 248;
    let varp_u8: u8 = 249;
    let varp_u16: u16 = 249;
    let varp_u32: u32 = 249;
    let varp_u64: u64 = 249;
    let varq_u8: u8 = 250;
    let varq_u16: u16 = 250;
    let varq_u32: u32 = 250;
    let varq_u64: u64 = 250;
    let varr_u8: u8 = 251;
    let varr_u16: u16 = 251;
    let varr_u32: u32 = 251;
    let varr_u64: u64 = 251;
    let vars_u8: u8 = 252;
    let vars_u16: u16 = 252;
    let vars_u32: u32 = 252;
    let vars_u64: u64 = 252;
    let vart_u8: u8 = 253;
    let vart_u16: u16 = 253;
    let vart_u32: u32 = 253;
    let vart_u64: u64 = 253;
    let varu_u8: u8 = 254;
    let varu_u16: u16 = 254;
    let varu_u32: u32 = 254;
    let varu_u64: u64 = 254;
    let varv_u8: u8 = 255;
    let varv_u16: u16 = 255;
    let varv_u32: u32 = 255;
    let varv_u64: u64 = 255;
    let varw_u8: u8 = 56;
    let varw_u16: u16 = 256;
    let varw_u32: u32 = 256;
    let varw_u64: u64 = 256;
    let varx_u8: u8 = 57;
    let varx_u16: u16 = 257;
    let varx_u32: u32 = 257;
    let varx_u64: u64 = 257;
    let vary_u8: u8 = 58;
    let vary_u16: u16 = 258;
    let vary_u32: u32 = 258;
    let vary_u64: u64 = 258;
    let varz_u8: u8 = 59;
    let varz_u16: u16 = 259;
    let varz_u32: u32 = 259;
    let varz_u64: u64 = 259;
    let vara_u8: u8 = 60;
    let vara_u16: u16 = 260;
    let vara_u32: u32 = 260;
    let vara_u64: u64 = 260;
    let varb_u8: u8 = 61;
    let varb_u16: u16 = 261;
    let varb_u32: u32 = 261;
    let varb_u64: u64 = 261;
    let varc_u8: u8 = 62;
    let varc_u16: u16 = 262;
    let varc_u32: u32 = 262;
    let varc_u64: u64 = 262;
    let vard_u8: u8 = 63;
    let vard_u16: u16 = 263;
    let vard_u32: u32 = 263;
    let vard_u64: u64 = 263;
    let vare_u8: u8 = 64;
    let vare_u16: u16 = 264;
    let vare_u32: u32 = 264;
    let vare_u64: u64 = 264;
    let varf_u8: u8 = 65;
    let varf_u16: u16 = 265;
    let varf_u32: u32 = 265;
    let varf_u64: u64 = 265;
    let varg_u8: u8 = 66;
    let varg_u16: u16 = 266;
    let varg_u32: u32 = 266;
    let varg_u64: u64 = 266;
    let varh_u8: u8 = 67;
    let varh_u16: u16 = 267;
    let varh_u32: u32 = 267;
    let varh_u64: u64 = 267;
    let vari_u8: u8 = 68;
    let vari_u16: u16 = 268;
    let vari_u32: u32 = 268;
    let vari_u64: u64 = 268;
    let varj_u8: u8 = 69;
    let varj_u16: u16 = 269;
    let varj_u32: u32 = 269;
    let varj_u64: u64 = 269;
    let vark_u8: u8 = 70;
    let vark_u16: u16 = 270;
    let vark_u32: u32 = 270;
    let vark_u64: u64 = 270;
    let varl_u8: u8 = 71;
    let varl_u16: u16 = 271;
    let varl_u32: u32 = 271;
    let varl_u64: u64 = 271;
    let varm_u8: u8 = 72;
    let varm_u16: u16 = 272;
    let varm_u32: u32 = 272;
    let varm_u64: u64 = 272;
    let varn_u8: u8 = 73;
    let varn_u16: u16 = 273;
    let varn_u32: u32 = 273;
    let varn_u64: u64 = 273;
    let varo_u8: u8 = 74;
    let varo_u16: u16 = 274;
    let varo_u32: u32 = 274;
    let varo_u64: u64 = 274;
    let varp_u8: u8 = 75;
    let varp_u16: u16 = 275;
    let varp_u32: u32 = 275;
    let varp_u64: u64 = 275;
    let varq_u8: u8 = 76;
    let varq_u16: u16 = 276;
    let varq_u32: u32 = 276;
    let varq_u64: u64 = 276;
    let varr_u8: u8 = 77;
    let varr_u16: u16 = 277;
    let varr_u32: u32 = 277;
    let varr_u64: u64 = 277;
    let vars_u8: u8 = 78;
    let vars_u16: u16 = 278;
    let vars_u32: u32 = 278;
    let vars_u64: u64 = 278;
    let vart_u8: u8 = 79;
    let vart_u16: u16 = 279;
    let vart_u32: u32 = 279;
    let vart_u64: u64 = 279;
    let varu_u8: u8 = 20;
    let varu_u16: u16 = 280;
    let varu_u32: u32 = 280;
    let varu_u64: u64 = 280;
    let varv_u8: u8 = 28;
    let varv_u16: u16 = 281;
    let varv_u32: u32 = 281;
    let varv_u64: u64 = 281;
    let varw_u8: u8 = 82;
    let varw_u16: u16 = 282;
    let varw_u32: u32 = 282;
    let varw_u64: u64 = 282;
    let varx_u8: u8 = 83;
    let varx_u16: u16 = 283;
    let varx_u32: u32 = 283;
    let varx_u64: u64 = 283;
    let vary_u8: u8 = 84;
    let vary_u16: u16 = 284;
    let vary_u32: u32 = 284;
    let vary_u64: u64 = 284;
    let varz_u8: u8 = 85;
    let varz_u16: u16 = 285;
    let varz_u32: u32 = 285;
    let varz_u64: u64 = 285;
    let vara_u8: u8 = 86;
    let vara_u16: u16 = 286;
    let vara_u32: u32 = 286;
    let vara_u64: u64 = 286;
    let varb_u8: u8 = 87;
    let varb_u16: u16 = 287;
    let varb_u32: u32 = 287;
    let varb_u64: u64 = 287;
    let varc_u8: u8 = 28;
    let varc_u16: u16 = 288;
    let varc_u32: u32 = 288;
    let varc_u64: u64 = 288;
    let vard_u8: u8 = 29;
    let vard_u16: u16 = 289;
    let vard_u32: u32 = 289;
    let vard_u64: u64 = 289;
    let vare_u8: u8 = 20;
    let vare_u16: u16 = 290;
    let vare_u32: u32 = 290;
    let vare_u64: u64 = 290;
    let varf_u8: u8 = 21;
    let varf_u16: u16 = 291;
    let varf_u32: u32 = 291;
    let varf_u64: u64 = 291;
    let varg_u8: u8 = 22;
    let varg_u16: u16 = 292;
    let varg_u32: u32 = 292;
    let varg_u64: u64 = 292;
    let varh_u8: u8 = 23;
    let varh_u16: u16 = 293;
    let varh_u32: u32 = 293;
    let varh_u64: u64 = 293;
    let vari_u8: u8 = 24;
    let vari_u16: u16 = 294;
    let vari_u32: u32 = 294;
    let vari_u64: u64 = 294;
    let varj_u8: u8 = 25;
    let varj_u16: u16 = 295;
    let varj_u32: u32 = 295;
    let varj_u64: u64 = 295;
    let vark_u8: u8 = 26;
    let vark_u16: u16 = 296;
    let vark_u32: u32 = 296;
    let vark_u64: u64 = 296;
    let varl_u8: u8 = 27;
    let varl_u16: u16 = 297;
    let varl_u32: u32 = 297;
    let varl_u64: u64 = 297;
    let varm_u8: u8 = 28;
    let varm_u16: u16 = 298;
    let varm_u32: u32 = 298;
    let varm_u64: u64 = 298;
    let varn_u8: u8 = 29;
    let varn_u16: u16 = 299;
    let varn_u32: u32 = 299;
    let varn_u64: u64 = 299;
    let varo_u8: u8 = 30;
    let varo_u16: u16 = 300;
    let varo_u32: u32 = 300;
    let varo_u64: u64 = 300;
    let s1 = Struct1 { field1: varb_u8, field2: varb_u16, field3: varb_u32, field4: varb_u64 };
    let a = func_struct1(s1);
    let b = func_struct2(Struct2 { field1: varc_u8, field2: varc_u16, field3: varc_u32, field4: varc_u64 });
    let c = func_struct3(Struct3 { field1: vard_u8, field2: vard_u16, field3: vard_u32, field4: vard_u64 });
    let d = func_struct4(Struct4 { field1: vare_u8, field2: vare_u16, field3: vare_u32, field4: vare_u64 });
    let e = func_struct5(Struct5 { field1: varf_u8, field2: varf_u16, field3: varf_u32, field4: varf_u64 });
    let f = func_struct6(Struct6 { field1: varg_u8, field2: varg_u16, field3: varg_u32, field4: varg_u64 });
    let g = func_struct7(Struct7 { field1: varh_u8, field2: varh_u16, field3: varh_u32, field4: varh_u64 });
    let h = func_struct8(Struct8 { field1: vari_u8, field2: vari_u16, field3: vari_u32, field4: vari_u64 });
    let i = func_struct9(Struct9 { field1: varj_u8, field2: varj_u16, field3: varj_u32, field4: varj_u64 });
    let j = Struct10 { field1: vark_u8, field2: vark_u16, field3: vark_u32, field4: vark_u64 };
    let k = Enum11::Variant1(varl_u8);
    let l = func_enum12(Enum12::Variant1(varm_u8));
    let m = func_enum13(Enum13::Variant1(varn_u8));
    let n = func_enum14(Enum14::Variant1(varo_u8));
    let o = func_enum15(Enum15::Variant1(varp_u8));
    let p = func_enum16(Enum16::Variant1(varq_u8));
    let q = func_enum17(Enum17::Variant1(varr_u8));
    let r = func_enum18(Enum18::Variant1(vars_u8));
    let s = func_enum19(Enum19::Variant1(vart_u8));
    let t = func_enum20(Enum20::Variant1(varu_u8));
    let u = func21(varv_u64, varv_u64);
    let v = func22(varw_u8, varw_u8);
    let w = func23(varx_u16, varx_u16);
    let x = func24(vary_u32, vary_u32);
    let y = func25(varz_u64, varz_u64);
    let z = func26(vara_u8, vara_u8);
    let func27 = func27(varb_u16, varb_u16);
    let func28 = func28(varc_u32, varc_u32);
    let func29 = func29(vard_u16, vard_u16);
    let func30 = func30(vare_u32, vare_u32);
}
