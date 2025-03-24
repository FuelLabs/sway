library;

#[deprecated(note = true)]
pub fn not_ok() {}

pub fn call_not_ok() {
    not_ok();
}