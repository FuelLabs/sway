script;

fn main() {
    let mut a = [u64; 0];
    a[0] = 1;


    let mut b = [[u64; 1]; 1];
    b[0][1] = 1;


    b[1][0] = 1;


    a[0] = return;
}
