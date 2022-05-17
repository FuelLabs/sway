library logging;


pub fn log<T>(value: T) {
    if ! __is_reference_type::<T>() {
        asm(r1: value) {
            log r1 zero zero zero;
        }
    } else {
        let size = __size_of::<T>();
        asm(r1: value, r2: size) {
            logd zero zero r1 r2;
        };
    }
}
