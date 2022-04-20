contract;

use std::{
    address::Address,
    context::*,
    assert::assert,
    contract_id::ContractId,
    token::*
};

abi LiquidityPool {
    fn deposit(recipient: Address);
    fn withdraw(recipient: Address);
}

const BASE_TOKEN = 0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dcdc30c;

impl LiquidityPool for Contract {
    fn deposit(recipient: Address) {
        assert((msg_asset_id()).into() == BASE_TOKEN);
        assert(msg_amount() > 0);

        // Mint two times the amount.
        let amount_to_mint = msg_amount() * 2;

        // Mint some LP token based upon the amount of the base token.
        mint_to_address(amount_to_mint, recipient);
    }

    fn withdraw(recipient: Address) {
        assert((msg_asset_id()).into() == contract_id());
        assert(msg_amount() > 0);

        // Amount to withdraw.
        let amount_to_transfer = msg_amount() / 2;

        // Transfer base token to recipient.
        transfer_to_output(amount_to_transfer, BASE_TOKEN, recipient);
    }
}
