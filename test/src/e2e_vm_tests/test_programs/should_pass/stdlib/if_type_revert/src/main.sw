script;

fn main() -> u64 {
    if true {
        std::revert::revert(0) 
    } else {
        42
    }
}