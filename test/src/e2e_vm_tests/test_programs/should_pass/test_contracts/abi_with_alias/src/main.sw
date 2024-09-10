contract;

type AliasedTuple = (u64, u64);

abi MyContract {
    fn tuple(arg1: (u64, u64)); // Inline
    fn aliased_tuple(arg1: AliasedTuple); // Alias
}

impl MyContract for Contract {
    fn tuple(_arg1: (u64, u64)) {
    }
    fn aliased_tuple(arg1: AliasedTuple) {
    }
}
