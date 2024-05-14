contract;

use std::{
    asset::{
        mint_to,
        transfer,
    },
    call_frames::msg_asset_id,
    constants::DEFAULT_SUB_ID,
    context::msg_amount,
    hash::*,
};

abi LiquidityPool {
    fn deposit(recipient: Address);
    fn withdraw(recipient: Address);
}

const BASE_ASSET: AssetId = AssetId::from(0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dcdc30c);

impl LiquidityPool for Contract {
    fn deposit(recipient: Address) {
        assert(msg_asset_id() == BASE_ASSET);
        assert(msg_amount() > 0);

        // Mint two times the amount.
        let amount_to_mint = msg_amount() * 2;

        // Mint some LP assets based upon the amount of the base asset.
        mint_to(Identity::Address(recipient), DEFAULT_SUB_ID, amount_to_mint);
    }

    fn withdraw(recipient: Address) {
        let asset_id = AssetId::default();
        assert(msg_asset_id() == asset_id);
        assert(msg_amount() > 0);

        // Amount to withdraw.
        let amount_to_transfer = msg_amount() / 2;

        // Transfer base asset to recipient.
        transfer(Identity::Address(recipient), BASE_ASSET, amount_to_transfer);
    }
}
