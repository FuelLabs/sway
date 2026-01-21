library;

fn reassignment() {
    let mut a = [0; 0];
    a[0] = 1;

    let mut b = [[0; 1]; 1];
    b[0][1] = 1;

    b[1][0] = 1;

    a[0] = return;
}

#[test]
fn test() {
    let _ = reassignment();
}
