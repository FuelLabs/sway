library reentrancy_target_abi;

abi Target {
    fn can_be_reentered() -> bool;
    fn reentrant_proof() -> bool;
}

