library storage_access_abi;

pub struct S {
    x: u64,
    y: u64,
    z: b256,
    t: T,
}

pub struct T {
    x: u64,
    y: u64,
    z: b256,
}

abi StorageAccess {
    // Setters
    fn set_x(x: u64);
    fn set_y(y: b256);
    fn set_s(s: S);
    fn set_s_dot_t(t: T);
    fn set_s_dot_x(x: u64);
    fn set_s_dot_y(y: u64);
    fn set_s_dot_z(z: b256);
    fn set_s_dot_t_dot_x(a: u64);
    fn set_s_dot_t_dot_y(b: u64);
    fn set_s_dot_t_dot_z(c: b256);

    // Getters
    fn get_x() -> u64;
    fn get_y() -> b256;
    fn get_s() -> S;
    fn get_s_dot_x() -> u64;
    fn get_s_dot_y() -> u64;
    fn get_s_dot_z() -> b256;
    fn get_s_dot_t() -> T;
    fn get_s_dot_t_dot_x() -> u64;
    fn get_s_dot_t_dot_y() -> u64;
    fn get_s_dot_t_dot_z() -> b256;
}
