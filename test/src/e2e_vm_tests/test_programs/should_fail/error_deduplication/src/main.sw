contract;

fn foo() -> u64 {
    let y = if x { 42 } else { 43 };
    if x { y } else { y }
}

fn foo() {}
fn foo() {}