library;

// 24 bytes = 3 x u64
pub struct Struct24 {
    pub a: u64,
    pub b: u64,
    pub c: u64,
}

pub const STRUCT24_DEFAULT: Struct24 = Struct24 {
    a: 0,
    b: 0,
    c: 0,
};

// 32 bytes = 4 x u64
pub struct Struct32 {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

pub const STRUCT32_DEFAULT: Struct32 = Struct32 {
    a: 0,
    b: 0,
    c: 0,
    d: 0,
};

// 40 bytes = 5 x u64
pub struct Struct40 {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    pub e: u64,
}

pub const STRUCT40_DEFAULT: Struct40 = Struct40 {
    a: 0,
    b: 0,
    c: 0,
    d: 0,
    e: 0,
};

// 48 bytes = Struct24 + Struct24
pub struct Struct48 {
    pub a: Struct24,
    pub b: Struct24,
}

pub const STRUCT48_DEFAULT: Struct48 = Struct48 {
    a: STRUCT24_DEFAULT,
    b: STRUCT24_DEFAULT,
};

// 56 bytes = Struct24 + Struct32
pub struct Struct56 {
    pub a: Struct24,
    pub b: Struct32,
}

pub const STRUCT56_DEFAULT: Struct56 = Struct56 {
    a: STRUCT24_DEFAULT,
    b: STRUCT32_DEFAULT,
};

// 72 bytes = Struct32 + Struct40
pub struct Struct72 {
    pub a: Struct32,
    pub b: Struct40,
}

pub const STRUCT72_DEFAULT: Struct72 = Struct72 {
    a: STRUCT32_DEFAULT,
    b: STRUCT40_DEFAULT,
};

// 88 bytes = Struct40 + Struct40 + u64
pub struct Struct88 {
    pub a: Struct40,
    pub b: Struct40,
    pub c: u64,
}

pub const STRUCT88_DEFAULT: Struct88 = Struct88 {
    a: STRUCT40_DEFAULT,
    b: STRUCT40_DEFAULT,
    c: 0,
};

// 96 bytes = Struct48 + Struct48
pub struct Struct96 {
    pub a: Struct48,
    pub b: Struct48,
}

pub const STRUCT96_DEFAULT: Struct96 = Struct96 {
    a: STRUCT48_DEFAULT,
    b: STRUCT48_DEFAULT,
};

// 184 bytes = Struct96 + Struct88
pub struct Struct184 {
    pub a: Struct96,
    pub b: Struct88,
}

pub const STRUCT184_DEFAULT: Struct184 = Struct184 {
    a: STRUCT96_DEFAULT,
    b: STRUCT88_DEFAULT,
};

// 200 bytes = Struct96 + Struct96 + u64
pub struct Struct200 {
    pub a: Struct96,
    pub b: Struct96,
    pub c: u64,
}

pub const STRUCT200_DEFAULT: Struct200 = Struct200 {
    a: STRUCT96_DEFAULT,
    b: STRUCT96_DEFAULT,
    c: 0,
};

// 224 bytes = Struct96 + Struct96 + Struct32
pub struct Struct224 {
    pub a: Struct96,
    pub b: Struct96,
    pub c: Struct32,
}

pub const STRUCT224_DEFAULT: Struct224 = Struct224 {
    a: STRUCT96_DEFAULT,
    b: STRUCT96_DEFAULT,
    c: STRUCT32_DEFAULT,
};

// 552 bytes = Struct224 + Struct224 + Struct96 + u64
pub struct Struct552 {
    pub a: Struct224,
    pub b: Struct224,
    pub c: Struct96,
    pub d: u64,
}

pub const STRUCT552_DEFAULT: Struct552 = Struct552 {
    a: STRUCT224_DEFAULT,
    b: STRUCT224_DEFAULT,
    c: STRUCT96_DEFAULT,
    d: 0,
};
