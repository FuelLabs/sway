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

// check: fn large_string<28c0f699>(s $MD: string<9>) -> string<9>
// check: entry(s: string<9>):
// check: ret string<9> $VAL

// check: fn small_string<80da70e2>(s $MD: string<7>) -> string<7>
// check: entry(s: string<7>):
// check: ret string<7> $VAL
