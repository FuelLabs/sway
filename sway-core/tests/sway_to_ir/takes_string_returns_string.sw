contract;

abi MyContract {
    fn small_string(s: str[7]) -> str[7];
    fn large_string(s: str[9]) -> str[9];
}

impl MyContract for Contract {
    fn small_string(s: str[7]) -> str[7] {
        s 
    }
    fn large_string(s: str[9]) -> str[9] {
        s 
    }
}
