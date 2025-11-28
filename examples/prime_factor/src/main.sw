    script;

    fn largest_prime_factor(n: u64) -> u64 {
        let mut n = n;
 

        let mut max_factor = 0;

        // remove factor 2 completely
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

        // if anything remains, it's a prime larger than sqrt(original n)
        if n > 1 {
            max_factor = n;
        }

        max_factor
    }

    fn main() {
        largest_prime_factor(9223372021822390277);
    }

    #[test]
    fn largest_prime_factor_large_input() {
        let result = largest_prime_factor(9223372021822390277);
        log(result);
    }
