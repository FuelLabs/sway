contract;

// ANCHOR: import
use reentrancy::reentrancy_guard;
// ANCHOR_END: import
abi Vault {
    fn deposit();
    fn withdraw();
}

impl Vault for Contract {
    // ANCHOR: guard
    fn deposit() {
        reentrancy_guard();

        // code
    }
    // ANCHOR_END: guard
    // ANCHOR: check
    fn withdraw() {
        // Step 1. Perform any state changes to update balance
        // Step 2. After all state changes make a call
    }
    // ANCHOR_END: check
}
