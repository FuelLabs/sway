fn is_prime(candidate: u64) -> bool {
    if candidate <= 1 {
        return false;
    }
    let mut divisor = 2;
    while divisor * divisor <= candidate {
        if candidate % divisor == 0 {
            return false;
        }
        divisor += 1;
    }
    true
}

fn main() {
    let max = 50_000;
    let mut count = 0;
    for n in 2..=max {
        if is_prime(n) {
            count += 1;
        }
    }
    println!("primes <= {}: {}", max, count);
}
