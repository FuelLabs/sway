script;

use core::*;

fn check_prime(n: u64) -> bool {
    if n == 0 || n == 1 {
        false
    } else {
        let mut is_not_prime = false;
        let mut i = 2;
        while i < n  {
            if n % i == 0 {
                is_not_prime = true;
                i = n; // break
            };
            i = i + 1;
        }

        !is_not_prime
    }
}

fn main() -> bool {
    assert(check_prime(64) == false);
    assert(check_prime(8) == false);
    assert(check_prime(7) == true);
    assert(check_prime(11) == true);
    assert(check_prime(13) == true);
    assert(check_prime(2) == true);
    assert(check_prime(3) == true);
    assert(check_prime(1) == false);
    assert(check_prime(0) == false);

    assert(check_prime(11) == check_prime(17));
    assert(check_prime(12) == false);
    assert(check_prime(18) == false);
    assert(check_prime(12) == check_prime(18));

    true
}
