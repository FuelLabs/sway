contract;

use std::{
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    constants::DEFAULT_SUB_ID,
    context::msg_amount,
    hash::*,
    token::{
        mint_to_address,
        transfer_to_address,
    },
};

abi LiquidityPool {
    fn deposit(recipient: Address);
    fn withdraw(recipient: Address);
}

const BASE_TOKEN: AssetId = AssetId {
    value: 0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dcdc30c,
};

impl LiquidityPool for Contract {
    fn deposit(recipient: Address) {
        assert(msg_asset_id() == BASE_TOKEN);
        assert(msg_amount() > 0);

        // Mint two times the amount.
        let amount_to_mint = msg_amount() * 2;

        // Mint some LP token based upon the amount of the base token.
        mint_to_address(recipient, DEFAULT_SUB_ID, amount_to_mint);
    }

    fn withdraw(recipient: Address) {
        let asset_id = AssetId::default();
        assert(msg_asset_id() == asset_id);
        assert(msg_amount() > 0);

        // Amount to withdraw.
        let amount_to_transfer = msg_amount() / 2;

        // Transfer base token to recipient.
        transfer_to_address(recipient, BASE_TOKEN, amount_to_transfer);
    }
}
