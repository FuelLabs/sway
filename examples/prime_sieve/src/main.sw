script;

fn main() -> u64 {
    let mut n: u64 = 30;
    let mut primes: [bool; 31] = [true; 31];
    primes[0] = false;
    primes[1] = false;

    let mut p: u64 = 2;
    while p * p <= n {
        if primes[p] {
            let mut i: u64 = p * p;
            while i <= n {
                primes[i] = false;
                i = i + p;
            }
        }
        p = p + 1;
    }

    let mut count: u64 = 0;
    let mut i: u64 = 2;
    while i <= n {
        if primes[i] {
            count = count + 1;
        }
        i = i + 1;
    }

    count
}
