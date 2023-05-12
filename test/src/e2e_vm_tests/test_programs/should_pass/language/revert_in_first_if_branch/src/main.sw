script;

fn main() {
    let cond = true;
    let _result = if cond == true { revert(42) } else { 0 };
}
