contract;

abi MyContract {
    fn test_function1(p1: u64);
    fn test_function2(mut p2: u64);
    fn test_function3(ref mut p3: u64);
    fn test_function4(ref p4: u64);
    fn test_function5(p5: u64);
    fn test_function6(p6: u64);
}

impl MyContract for Contract {
    fn test_function1(ref mut p1: u64) {

    }
    fn test_function2(mut p2: u64) {

    }
    fn test_function3(ref mut p3: u64) {

    }
    fn test_function4(ref p4: u64) {

    }
    fn test_function5(ref p5: u64) {

    }
    fn test_function6(mut p6: u64) {

    }
}

trait MyTrait {
    fn check_function1(q1: u64);
    fn check_function2(mut q2: u64);
    fn check_function3(ref mut q3: u64);
    fn check_function4(ref q4: u64);
    fn check_function5(q5: u64);
    fn check_function6(q6: u64);
}

struct S {

}

impl MyTrait for S {
    fn check_function1(ref mut q1: u64) {

    }
    fn check_function2(mut q2: u64) {

    }
    fn check_function3(ref mut q3: u64) {

    }
    fn check_function4(ref q4: u64) {

    }
    fn check_function5(ref q5: u64) {

    }
    fn check_function6(mut q6: u64) {

    }
}
