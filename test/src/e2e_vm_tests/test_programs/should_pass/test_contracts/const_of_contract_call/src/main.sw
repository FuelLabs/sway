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
    /* START BOOL */
    fn in_bool(v: bool) -> bool;
    /* END BOOL */

    /* START U8 */
    fn in_u8(v: u8) -> u8;
    /* END U8 */

    /* START U16 */
    fn in_u16(v: u16) -> u16;
    /* END U16 */

    /* START U32 */
    fn in_u32(v: u32) -> u32;
    /* END U32 */

    /* START U64 */
    fn in_u64(v: u64) -> u64;
    /* END U64 */

    /* START U256 */
    fn in_u256(v: u256) -> u256;
    /* END U256 */

    /* START B256 */
    fn in_b256(v: b256) -> b256;
    /* END B256 */

    /* START STR0 */
    fn in_str_0(v: str[0]) -> str[0];
    /* END STR0 */

    /* START STR1 */
    fn in_str_1(v: str[1]) -> str[1];
    /* END STR1 */

    /* START STR8 */
    fn in_str_8(v: str[8]) -> str[8];
    /* END STR8 */

    /* START STR16 */
    fn in_str_16(v: str[16]) -> str[16];
    /* END STR16 */

    /* START STR32 */
    fn in_str_32(v: str[32]) -> str[32];
    /* END STR32 */

    /* START ARRAY0 */
    fn in_array_0(v: [u64; 0]) -> [u64; 0];
    /* END ARRAY0 */

    /* START ARRAY1 */
    fn in_array_1(v: [u64; 1]) -> [u64; 1];
    /* END ARRAY1 */

    /* START ARRAY8 */
    fn in_array_8(v: [u64; 8]) -> [u64; 8];
    /* END ARRAY8 */

    /* START ARRAY16 */
    fn in_array_16(v: [u64; 16]) -> [u64; 16];
    /* END ARRAY16 */

    /* START ARRAY32 */
    fn in_array_32(v: [u64; 32]) -> [u64; 32];
    /* END ARRAY32 */

    /* START ARRAY64 */
    fn in_array_64(v: [u64; 64]) -> [u64; 64];
    /* END ARRAY64 */

    /* START TUPLE0 */
    fn in_tuple_0(v: ()) -> ();
    /* END TUPLE0 */

    /* START TUPLE1 */
    fn in_tuple_1(v: (u64,)) -> (u64,);
    /* END TUPLE1 */

    /* START TUPLE2 */
    fn in_tuple_2(v: (u64, u64)) -> (u64, u64);
    /* END TUPLE2 */

    /* START TUPLE3 */
    fn in_tuple_3(v: (u64, u64, u64)) -> (u64, u64, u64);
    /* END TUPLE3 */

    /* START TUPLE4 */
    fn in_tuple_4(v: (u64, u64, u64, u64)) -> (u64, u64, u64, u64);
    /* END TUPLE4 */

    /* START STRUCT_U64 */
    fn in_struct_u64(v: S1<u64>) -> S1<u64>;
    /* END STRUCT_U64 */
    
    /* START STRUCT_U64_U64 */
    fn in_struct_u64_u64(v: S2<u64, u64>) -> S2<u64, u64>;
    /* END STRUCT_U64_U64 */

    /* START STRUCT_U64_U64_U64 */
    fn in_struct_u64_u64_u64(v: S3<u64, u64, u64>) -> S3<u64, u64, u64>;
    /* END STRUCT_U64_U64_U64 */

    /* START ENUM_U64 */
    fn in_enum_u64(v: E1<u64>) -> E1<u64>;
    /* END ENUM_U64 */

    /* START ENUM_U64_U64 */
    fn in_enum_u64_u64(v: E2<u64, u64>) -> E2<u64, u64>;
    /* END ENUM_U64_U64 */

    /* START ENUM_U64_U64_U64 */
    fn in_enum_u64_u64_u64(v: E3<u64, u64, u64>) -> E3<u64, u64, u64>;
    /* END ENUM_U64_U64_U64 */
}

impl MyContract for Contract {
    /* START BOOL */
    fn in_bool(v: bool) -> bool { v }
    /* END BOOL */

    /* START U8 */
    fn in_u8(v: u8) -> u8 { v }
    /* END U8 */

    /* START U16 */
    fn in_u16(v: u16) -> u16 { v }
    /* END U16 */

    /* START U32 */
    fn in_u32(v: u32) -> u32 { v }
    /* END U32 */

    /* START U64 */
    fn in_u64(v: u64) -> u64 { v }
    /* END U64 */

    /* START U256 */
    fn in_u256(v: u256) -> u256 { v }
    /* END U256 */

    /* START B256 */
    fn in_b256(v: b256) -> b256 { v }
    /* END B256 */

    /* START STR0 */
    fn in_str_0(v: str[0]) -> str[0] { v }
    /* END STR0 */

    /* START STR1 */
    fn in_str_1(v: str[1]) -> str[1] { v }
    /* END STR1 */

    /* START STR8 */
    fn in_str_8(v: str[8]) -> str[8] { v }
    /* END STR8 */

    /* START STR16 */
    fn in_str_16(v: str[16]) -> str[16] { v }
    /* END STR16 */

    /* START STR32 */
    fn in_str_32(v: str[32]) -> str[32] { v }
    /* END STR32 */

    /* START ARRAY0 */
    fn in_array_0(v: [u64; 0]) -> [u64; 0] { v }
    /* END ARRAY0 */

    /* START ARRAY1 */
    fn in_array_1(v: [u64; 1]) -> [u64; 1] { v }
    /* END ARRAY1 */

    /* START ARRAY8 */
    fn in_array_8(v: [u64; 8]) -> [u64; 8]  { __log(v); v }
    /* END ARRAY8 */

    /* START ARRAY16 */
    fn in_array_16(v: [u64; 16]) -> [u64; 16] { v }
    /* END ARRAY16 */

    /* START ARRAY32 */
    fn in_array_32(v: [u64; 32]) -> [u64; 32] { v }
    /* END ARRAY32 */

    /* START ARRAY64 */
    fn in_array_64(v: [u64; 64]) -> [u64; 64] { v }
    /* END ARRAY64 */

    /* START TUPLE0 */
    fn in_tuple_0(v: ()) -> () { v }
    /* END TUPLE0 */

    /* START TUPLE1 */
    fn in_tuple_1(v: (u64,)) -> (u64,) { v }
    /* END TUPLE1 */

    /* START TUPLE2 */
    fn in_tuple_2(v: (u64, u64)) -> (u64, u64) { v }
    /* END TUPLE2 */

    /* START TUPLE3 */
    fn in_tuple_3(v: (u64, u64, u64)) -> (u64, u64, u64) { v }
    /* END TUPLE3 */

    /* START TUPLE4 */
    fn in_tuple_4(v: (u64, u64, u64, u64)) -> (u64, u64, u64, u64) { v }
    /* END TUPLE4 */

    /* START STRUCT_U64 */
    fn in_struct_u64(v: S1<u64>) -> S1<u64> { v }
    /* END STRUCT_U64 */
    
    /* START STRUCT_U64_U64 */
    fn in_struct_u64_u64(v: S2<u64, u64>) -> S2<u64, u64> { v }
    /* END STRUCT_U64_U64 */

    /* START STRUCT_U64_U64_U64 */
    fn in_struct_u64_u64_u64(v: S3<u64, u64, u64>) -> S3<u64, u64, u64> { v }
    /* END STRUCT_U64_U64_U64 */

    /* START ENUM_U64 */
    fn in_enum_u64(v: E1<u64>) -> E1<u64> { v }
    /* END ENUM_U64 */

    /* START ENUM_U64_U64 */
    fn in_enum_u64_u64(v: E2<u64, u64>) -> E2<u64, u64> { v }
    /* END ENUM_U64_U64 */

    /* START ENUM_U64_U64_U64 */
    fn in_enum_u64_u64_u64(v: E3<u64, u64, u64>) -> E3<u64, u64, u64> { v }
    /* END ENUM_U64_U64_U64 */
}

/* START BOOL */
#[test]
fn cost_of_in_bool() {
    let _ = abi(MyContract, CONTRACT_ID).in_bool(false);
}
/* END BOOL */

/* START U8 */
#[test]
fn cost_of_in_u8() {
    let _ = abi(MyContract, CONTRACT_ID).in_u8(0);
}
/* END U8 */

/* START U16 */
#[test]
fn cost_of_in_u16() {
    let _ = abi(MyContract, CONTRACT_ID).in_u16(0);
}
/* END U16 */

/* START U32 */
#[test]
fn cost_of_in_u32() {
    let _ = abi(MyContract, CONTRACT_ID).in_u32(0);
}
/* END U32 */

/* START U64 */
#[test]
fn cost_of_in_u64() {
    let _ = abi(MyContract, CONTRACT_ID).in_u64(0);
}
/* END U64 */

/* START U256 */
#[test]
fn cost_of_in_u256() {
    let _ = abi(MyContract, CONTRACT_ID).in_u256(0x0000000000000000000000000000000000000000000000000000000000000000u256);
}
/* END U256 */

/* START B256 */
#[test]
fn cost_of_in_b256() {
    let _ = abi(MyContract, CONTRACT_ID).in_b256(0x0000000000000000000000000000000000000000000000000000000000000000);
}
/* END B256 */

/* START STR0 */
#[test]
fn cost_of_in_str_0() {
    let _ = abi(MyContract, CONTRACT_ID).in_str_0(__to_str_array(""));
}
/* END STR0 */

/* START STR1 */
#[test]
fn cost_of_in_str_1() {
    let _ = abi(MyContract, CONTRACT_ID).in_str_1(__to_str_array("1"));
}
/* END STR1 */

/* START STR8 */
#[test]
fn cost_of_in_str_8() {
    let _ = abi(MyContract, CONTRACT_ID).in_str_8(__to_str_array("12345678"));
}
/* END STR8 */

/* START STR16 */
#[test]
fn cost_of_in_str_16() {
    let _ = abi(MyContract, CONTRACT_ID).in_str_16(__to_str_array("1234567890123456"));
}
/* END STR16 */

/* START STR32 */
#[test]
fn cost_of_in_str_32() {
    let _ = abi(MyContract, CONTRACT_ID).in_str_32(__to_str_array("12345678901234567890123456789012"));
}
/* END STR32 */

/* START ARRAY0 */
#[test]
fn cost_of_in_array_0() {
    let _ = abi(MyContract, CONTRACT_ID).in_array_0([]);
}
/* END ARRAY0 */

/* START ARRAY1 */
#[test]
fn cost_of_in_array_1() {
    let _ = abi(MyContract, CONTRACT_ID).in_array_1([0]);
}
/* END ARRAY1 */

/* START ARRAY8 */
#[test]
fn cost_of_in_array_8() {
    let _ = abi(MyContract, CONTRACT_ID).in_array_8([0, 0, 0, 0, 0, 0, 0, 0]);
}
/* END ARRAY8 */

/* START ARRAY16 */
#[test]
fn cost_of_in_array_16() {
    let _ = abi(MyContract, CONTRACT_ID).in_array_16([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}
/* END ARRAY16 */

/* START ARRAY32 */
#[test]
fn cost_of_in_array_32() {
    let _ = abi(MyContract, CONTRACT_ID).in_array_32([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}
/* END ARRAY32 */

/* START ARRAY64 */
#[test]
fn cost_of_in_array_64() {
    let _ = abi(MyContract, CONTRACT_ID).in_array_64([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}
/* END ARRAY64 */

/* START TUPLE0 */
#[test]
fn cost_of_in_tuple_0() {
    let _ = abi(MyContract, CONTRACT_ID).in_tuple_0(());
}
/* END TUPLE0 */

/* START TUPLE1 */
#[test]
fn cost_of_in_tuple_1() {
    let _ = abi(MyContract, CONTRACT_ID).in_tuple_1((0,));
}
/* END TUPLE1 */

/* START TUPLE2 */
#[test]
fn cost_of_in_tuple_2() {
    let _ = abi(MyContract, CONTRACT_ID).in_tuple_2((0,0));
}
/* END TUPLE2 */

/* START TUPLE3 */
#[test]
fn cost_of_in_tuple_3() {
    let _ = abi(MyContract, CONTRACT_ID).in_tuple_3((0,0,0));
}
/* END TUPLE3 */

/* START TUPLE4 */
#[test]
fn cost_of_in_tuple_4() {
    let _ = abi(MyContract, CONTRACT_ID).in_tuple_4((0,0,0,0));
}
/* END TUPLE4 */

/* START STRUCT_U64 */
#[test]
fn in_struct_u64() {
    let _ = abi(MyContract, CONTRACT_ID).in_struct_u64(S1 { a: 0 });
}
/* END STRUCT_U64 */

/* START STRUCT_U64_U64 */
#[test]
fn in_struct_u64_u64() {
    let _ = abi(MyContract, CONTRACT_ID).in_struct_u64_u64(S2 { a: 0, b: 0 });
}
/* END STRUCT_U64_U64 */

/* START STRUCT_U64_U64_U64 */
#[test]
fn in_struct_u64_u64_u64() {
    let _ = abi(MyContract, CONTRACT_ID).in_struct_u64_u64_u64(S3 { a: 0, b: 0, c: 0 });
}
/* END STRUCT_U64_U64_U64 */

/* START ENUM_U64 */
#[test]
fn in_enum_u64() {
    let _ = abi(MyContract, CONTRACT_ID).in_enum_u64(E1::A(0));
}
/* END ENUM_U64 */

/* START ENUM_U64_U64 */
#[test]
fn in_enum_u64_u64() {
    let _ = abi(MyContract, CONTRACT_ID).in_enum_u64_u64(E2::A(0));
}
/* END ENUM_U64_U64 */

/* START ENUM_U64_U64_U64 */
#[test]
fn in_enum_u64_u64_u64() {
    let _ = abi(MyContract, CONTRACT_ID).in_enum_u64_u64_u64(E3::A(0));
}
/* END ENUM_U64_U64_U64 */
