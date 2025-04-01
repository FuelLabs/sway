script;

const N: u64 = 10;

fn simple_break_test() {
    let mut i = 0;
    while true {
        if i >= N {
            break;
        }
        i += 1
    }
    assert(i == N);
}

fn simple_continue_test() {
    let mut i = 0;
    let mut sum = 0;
    while i < N {
        i += 1;
        if i % 2 == 0 {
            continue;
        }
        sum += 1;
    }
    assert(sum == N / 2);
}

fn break_and_continue_test() {
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    let mut n = 0;
    let mut sum1 = 0;
    let mut sum2 = 0;
    while true {
        if i >= N {
            break;
        }
        while true {
            if j >= N {
                break;
            }
            sum1 += i * N + j;
            j += 1;

            if j % 2 == 0 {
                continue;
            }

            while n < N {
                sum1 += n;
                n += 1;
                if sum1 > 50 {
                    break;
                }
            }
        }

        while true {
            if k >= N {
                break;
            }
            sum1 += i * N + k;
            k += 1;

            if k % 2 == 0 {
                continue;
            }

            sum1 *= 2;
        }
        i += 1;

        if i % 2 == 0 {
            continue;
        }

        sum1 += 1;
        sum2 += 1;
    }

    assert(sum1 == 3072);
    assert(sum2 == 5);
}

fn main() -> bool {
    simple_break_test();
    simple_continue_test();
    break_and_continue_test();

    true
}
