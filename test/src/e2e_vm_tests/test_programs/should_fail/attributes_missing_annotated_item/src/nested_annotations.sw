// TODO: Adjust this test once https://github.com/FuelLabs/sway/issues/6932 is implemented.
library;

pub fn fa() {
    /// This is an outer doc comment.
}

pub fn fb() {
    //! This is an inner doc comment.
}

pub fn fc() {
    #[allow(dead_code)]
}