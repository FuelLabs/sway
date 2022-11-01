script;

// Explicit returns from each arm of a match expression.  Was causing mistyped dead IR to be
// generated.

fn main() -> bool {
    match true {
        true => {
            return true;
        },
        false => {
            return false;
        }
    }
}
