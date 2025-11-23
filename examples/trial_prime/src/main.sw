script;

fn is_prime(candidate: u64) -> bool {
    if candidate <= 1 {
        return false;
    }
    let mut divisor: u64 = 2;
    while divisor * divisor <= candidate {
        if candidate % divisor == 0 {
            return false;
        }
        divisor = divisor + 1;
    }
    true
}

fn main() -> u64 {
    let mut count: u64 = 0;
    let max: u64 = 5000_000;
    let mut n: u64 = 2;
    while n <= max {
        if is_prime(n) {
            count = count + 1;
        }
        n = n + 1;
    }
    count
}
