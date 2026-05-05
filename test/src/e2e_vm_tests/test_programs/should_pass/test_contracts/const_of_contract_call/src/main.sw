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
    

    

    

    

    

    

    

    

    

    

    

    

    /* START ARRAY0 */
    fn in_array_0(v: [u64; 0]) -> [u64; 0];
    /* END ARRAY0 */

    

    

    

    

    

    

    

    

    

    

    
    
    

    

    

    

    

    

    
}

impl MyContract for Contract {
    

    

    

    

    

    

    

    

    

    

    

    

    /* START ARRAY0 */
    fn in_array_0(v: [u64; 0]) -> [u64; 0] { v }
    /* END ARRAY0 */

    

    

    

    

    

    

    

    

    

    

    
    
    

    

    

    

    

    

    
}

























/* START ARRAY0 */
#[test]
fn isolated_cost_of_in_array_0() {
    let _ = abi(MyContract, CONTRACT_ID).in_array_0([]);
}
/* END ARRAY0 */



































