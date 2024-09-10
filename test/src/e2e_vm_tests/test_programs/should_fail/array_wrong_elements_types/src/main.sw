script;

fn vec<T>() -> Vec<T> {
    Vec::new()
}

fn main() {
    // unexpected u16
    let a: [u8;1] = [1u16];

    // unexpected u16
    let _ = [1u8, 1u16];
    let a = [1, 2u8, 3u16, 4u32, 5u64];

    // unexpected u8
    let _ = [1, 1u16, a[0]];

    // unexpected string slice
    let _ = [1, "", 1u8, 1u16];

    // unexpected u8
    let _ = [return, 1u8, 1u16];

    // unexpected u16
    let _ = [1u8, return, 1u16];

    // unexpected u16
    let _ = [1, return, 1u8, 1u16];

    // unexpected str
    let _ = [1, "", 1u16];
    let _ = [1, 2, "hello"];
    let _ = [1, return, "", 1u16];
    let _ = [1, "", return, 1u16];

    // unexpected Vec<u16>
    let _ = [Vec::new(), vec::<u8>(), vec::<u16>()];
    
    // unexpected Option<u8>
    let a = [None, Some(1), Some(1u8)];
    let _b: Option<u16> = a[1];

    // unexpected u8
    let a = [8, 256u16, 8u8];
    let b: u32 = a[2];

    // Should not warn or error
    let _ : [u8 ; 0] = [];
}
