library reentrancy_target_abi;

abi Target {
    fn reentrancy_detected() -> bool;
    fn reentrance_denied();
    fn cross_function_reentrance_denied();
    fn guarded_function_is_callable() -> bool;
}
