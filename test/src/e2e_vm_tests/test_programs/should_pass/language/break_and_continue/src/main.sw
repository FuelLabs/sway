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

            if j % 2 == 0 {
                continue;
            }

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

            if k % 2 == 0 {
                continue;
            }

            sum1 *= 2;

        }
        i += 1;

        if i % 3 == 0 {
            continue;
        }

        sum1 *= 2;
        sum2 *= 2;

    }

    sum1 + sum2 // = 281250103296, Validated against Rust
}

fn main() -> u64 {
    break_test()
}
