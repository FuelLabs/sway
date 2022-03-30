library reentrancy_target_abi;

abi Target {
    fn reentrance_denied();
    fn reentrancy_detected() -> bool;
}
