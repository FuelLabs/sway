script;

const J = 10;
const K = 20;
const I = 30;
const N = 100;

fn break_test() -> u64 {
    let m = 5;
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    let mut n = 0;
    let mut sum1 = 0;
    let mut sum2 = 0;
    while true {
        if i >= I {
            break;
        }
        while true {
            if j >= J {
                break;
            }
            sum1 += i * J + j;
            j += 1;

            while n < N {
                sum1 += n;
                n += 2;
                if sum1 > 100 {
                    break;
                }
            }
        }

        while true {
            if k >= K {
                break;
            }
            sum1 += i * K + k;
            k += 1;
        }
        i += 1;
    }

    sum1 + sum2 // = 615, Validated against Rust
}

fn main() -> u64 {
    break_test()
}
