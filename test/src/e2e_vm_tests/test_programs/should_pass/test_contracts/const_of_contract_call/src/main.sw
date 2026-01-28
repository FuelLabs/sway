contract;

struct S1<A> {
    #[allow(dead_code)]
    a: A
}
struct S2<A, B> { 
    #[allow(dead_code)]
    a: A,
    #[allow(dead_code)]
    b: B
}
struct S3<A, B, C> {
    #[allow(dead_code)]
    a: A,
    #[allow(dead_code)]
    b: B,
    #[allow(dead_code)]
    c: C
}

enum E1<A> { A: A }
enum E2<A, B> { A: A, B: B }
enum E3<A, B, C> { A: A, B: B, C: C }

abi MyContract {
    

    

    

    

    

    

    

    

    

    

    

    

    

    

    

    

    /* START ARRAY32 */
    fn in_array_32(v: [u64; 32]) -> [u64; 32];
    /* END ARRAY32 */

    

    

    

    

    

    

    
    
    

    

    

    

    
}

impl MyContract for Contract {
    

    

    

    

    

    

    

    

    

    

    

    

    

    

    

    

    /* START ARRAY32 */
    fn in_array_32(v: [u64; 32]) -> [u64; 32] { v }
    /* END ARRAY32 */

    

    

    

    

    

    

    
    
    

    

    

    

    
}

































/* START ARRAY32 */
#[test]
fn isolated_cost_of_in_array_32() {
    let _ = abi(MyContract, CONTRACT_ID).in_array_32([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}
/* END ARRAY32 */
























