fn largest_prime_factor(mut n: u64) -> u64 {
    if n <= 1 {
        return n;
    }

    let mut max_factor = 0;

    while n % 2 == 0 {
        max_factor = 2;
        n /= 2;
    }

    let mut i = 3;
    while i * i <= n {
        while n % i == 0 {
            max_factor = i;
            n /= i;
        }
        i += 2;
    }

    if n > 1 {
        max_factor = n;
    }

    max_factor
}

fn main() {
    let n: u64 = 9_223_372_021_822_390_277;
    let result = largest_prime_factor(n);
    println!("largest prime factor of {n} = {result}");
}
