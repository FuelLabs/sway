library utils;

pub fn vec_from(vals: [u32; 3]) -> Vec<u32> {
    let mut vec = Vec::new();
    vec.push(vals[0]);
    vec.push(vals[1]);
    vec.push(vals[2]);
    vec
}

pub fn get_an_option<T>() -> Option<T> {
    Option::None
}
