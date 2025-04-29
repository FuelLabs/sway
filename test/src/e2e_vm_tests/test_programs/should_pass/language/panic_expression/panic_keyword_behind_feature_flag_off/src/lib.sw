library;

pub fn panic_keyword_behind_feature_flag_off() {
    let mut panic = "This is a panic string in a panic variable.";
    panic = "This is a new panic string in a panic variable.";
    let _ = panic;
    panic;
    poke(panic);
}

pub fn poke<T>(_t: T) { }